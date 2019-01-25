use rayon::{current_num_threads, Scope};
use rayon_adaptive::atomiclist::{AtomicLink, AtomicList};
use rayon_adaptive::fuse_slices;
use rayon_adaptive::prelude::*;
use rayon_adaptive::smallchannel::{small_channel, SmallSender};
use rayon_adaptive::utils::powers;
use std::cmp::min;
use std::iter::{once, repeat};

/// by default, min block size is log(n)
fn default_min_block_size(n: usize) -> usize {
    (n as f64).log(2.0).floor() as usize
}

/// by default, max block size is sqrt(n)
fn default_max_block_size(n: usize) -> usize {
    ((n as f64).sqrt() * 10.0f64).ceil() as usize
}

/// compute a block size with the given function.
/// this allows us to ensure we enforce important bounds on sizes.
fn compute_size<F: Fn(usize) -> usize>(n: usize, sizing_function: F) -> usize {
    let p = current_num_threads();
    std::cmp::max(min(n / (2 * p), sizing_function(n)), 1)
}

/// We are going to do one big fold operation in order to compute
/// the final result.
/// Sometimes we fold on some input but sometimes we also fold
/// on intermediate outputs.
/// Having an enumerated type enables to conveniently iterate on both types.
enum FoldElement<I, O2> {
    Input(I),
    Output(O2),
}

fn fold_with_help<I, O1, O2, ID2, FOLD1, FOLD2, RET>(
    input: I,
    o1: O1,
    fold1: FOLD1,
    id2: ID2,
    fold2: FOLD2,
    retrieve: RET,
) -> O1
where
    I: DivisibleIntoBlocks,
    ID2: Fn() -> O2 + Sync + Send + Copy,
    O1: Send + Sync + Copy,
    O2: Send + Sync,
    FOLD1: Fn(O1, I, usize) -> (O1, I) + Sync + Send + Copy,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy,
    RET: Fn(O1, O2) -> O1 + Sync + Send + Copy,
{
    let input_length = input.base_length();
    let macro_block_size = compute_size(input_length, default_max_block_size);
    let nano_block_size = compute_size(input_length, default_min_block_size);
    let stolen_stuffs: &AtomicList<(Option<O2>, Option<I>)> = &AtomicList::new();
    rayon::scope(|s| {
        input
            .chunks(repeat(macro_block_size))
            .flat_map(|chunk| {
                once(FoldElement::Input(chunk)).chain(stolen_stuffs.iter().flat_map(|(o2, i)| {
                    o2.map(|o| FoldElement::Output(o))
                        .into_iter()
                        .chain(i.map(|i| FoldElement::Input(i)).into_iter())
                }))
            })
            .fold(o1, |o1, element| match element {
                FoldElement::Input(i) => {
                    master_work(s, o1, i, fold1, id2, fold2, stolen_stuffs, nano_block_size)
                }
                FoldElement::Output(o2) => retrieve(o1, o2),
            })
    })
}

fn spawn_stealing_task<'scope, I, ID2, O2, FOLD2>(
    scope: &Scope<'scope>,
    id2: ID2,
    fold2: FOLD2,
    initial_size: usize,
) -> SmallSender<AtomicLink<(Option<O2>, Option<I>)>>
where
    I: DivisibleIntoBlocks + 'scope,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'scope,
    O2: Send + Sync + 'scope,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'scope,
{
    let (sender, receiver) = small_channel();
    scope.spawn(move |s| {
        let stolen_input: Option<AtomicLink<(Option<O2>, Option<I>)>> = receiver.recv();
        if stolen_input.is_none() {
            return;
        }
        slave_work(s, stolen_input.unwrap(), id2, fold2, initial_size)
    });
    sender
}

fn master_work<'scope, I, O1, O2, FOLD1, ID2, FOLD2>(
    scope: &Scope<'scope>,
    init: O1,
    input: I,
    fold: FOLD1,
    id2: ID2,
    fold2: FOLD2,
    stolen_stuffs: &AtomicList<(Option<O2>, Option<I>)>,
    initial_size: usize,
) -> O1
where
    I: DivisibleIntoBlocks + 'scope,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'scope,
    O1: Send + Sync + Copy,
    O2: Send + Sync + 'scope,
    FOLD1: Fn(O1, I, usize) -> (O1, I) + Sync + Send + Copy,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'scope,
{
    let mut input = input;
    let mut current_output = init;
    loop {
        let sender = spawn_stealing_task(scope, id2, fold2, initial_size);
        // let's work sequentially until stolen
        match powers(initial_size)
            .take_while(|_| !sender.receiver_is_waiting())
            .try_fold((current_output, input), |(output, input), size| {
                let checked_size = min(input.base_length(), size); //TODO: remove all these mins
                if checked_size > 0 {
                    Ok(fold(output, input, checked_size))
                } else {
                    Err(output)
                }
            }) {
            Ok((output, remaining_input)) => {
                if remaining_input.base_length() > initial_size {
                    let (my_half, his_half) = remaining_input.divide();
                    if his_half.base_length() > 0 {
                        let stolen_node = stolen_stuffs.push_front((None, Some(his_half)));
                        sender.send(stolen_node);
                    }
                    input = my_half;
                    current_output = output;
                } else {
                    let length = remaining_input.base_length();
                    return fold(output, remaining_input, length).0;
                }
            }
            Err(output) => return output,
        }
    }
}

//TODO: we could maybe avoid code duplication between master and slave with a dummy head of the
//list for the master
fn slave_work<'scope, I, O2, ID2, FOLD2>(
    scope: &Scope<'scope>,
    node: AtomicLink<(Option<O2>, Option<I>)>,
    id2: ID2,
    fold2: FOLD2,
    initial_size: usize,
) where
    I: DivisibleIntoBlocks + 'scope,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'scope,
    O2: Send + Sync + 'scope,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'scope,
{
    let mut input = node.take().unwrap().1.unwrap();
    let mut o2 = id2();
    loop {
        let sender = spawn_stealing_task(scope, id2, fold2, initial_size);
        // let's work sequentially until stolen
        match powers(initial_size)
            .take_while(|_| !sender.receiver_is_waiting() && !node.requested())
            .try_fold((o2, input), |(output2, input), size| {
                let checked_size = min(input.base_length(), size); //TODO: remove all these mins
                if checked_size > 0 {
                    Ok(fold2(output2, input, checked_size))
                } else {
                    Err(output2)
                }
            }) {
            Ok((output2, remaining_input)) => {
                if node.requested() {
                    // retrieval operations are prioritized over steal ops
                    node.replace((Some(output2), Some(remaining_input)));
                    return;
                } else {
                    // check if enough is left
                    let length = remaining_input.base_length();
                    if length > initial_size {
                        let (my_half, his_half) = remaining_input.divide();
                        // TODO: have an empty method
                        if his_half.base_length() > 0 {
                            let stolen_node = (&node).split((None, Some(his_half)));
                            sender.send(stolen_node)
                        }
                        input = my_half;
                        o2 = output2;
                    } else {
                        // just fold it locally
                        node.replace((Some(fold2(output2, remaining_input, length).0), None));
                        return;
                    }
                }
            }
            Err(output2) => {
                node.replace((Some(output2), None));
                return;
            }
        }
    }
}

const SIZE: usize = 1000_000;

fn main() {
    //TODO: also provide the nicer fold api
    (1..=4).for_each(|number_of_threads| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build()
            .expect("Thread pool build failed");
        let mut input_vector = vec![1.0; SIZE];
        let expected_result: Vec<_> = vec![1.0; SIZE];
        let start = time::precise_time_ns();
        pool.scope(|s| {
            fold_with_help(
                input_vector.as_mut_slice(),
                1.0,
                |last_elem_prev_slice, remaining_slice, limit| {
                    let (todo, remaining) = remaining_slice.divide_at(limit);
                    (
                        todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                            *e = *e * c;
                            e.clone()
                        }),
                        remaining,
                    )
                },
                || None,
                |possible_previous_slice: Option<&mut [f64]>, input, limit| {
                    let last_elem_prev_slice = possible_previous_slice
                        .as_ref()
                        .and_then(|c| c.last().cloned())
                        .unwrap_or(1.0);
                    let (todo, remaining) = input.divide_at(limit);
                    todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                        *e = *e * c;
                        e.clone()
                    });
                    (
                        possible_previous_slice
                            .map(|previous| fuse_slices(previous, todo))
                            .or_else(move || Some(todo)),
                        remaining,
                    )
                },
                |last_num, dirty_slice| {
                    if let Some(retrieved_slice) = dirty_slice {
                        let last_slice_num = retrieved_slice.last().cloned().unwrap();
                        s.spawn(move |_| {
                            retrieved_slice
                                .into_adapt_iter()
                                .for_each(|e| *e *= last_num)
                        });
                        last_num * last_slice_num
                    } else {
                        last_num
                    }
                },
            );
        });
        let end = time::precise_time_ns();
        let time_taken_ms = ((end - start) as f64) / (1e6 as f64);
        assert_eq!(input_vector, expected_result);

        println!("{}, {}", time_taken_ms, number_of_threads);
    });
}
