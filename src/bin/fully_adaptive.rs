use rayon::current_num_threads;
use rayon::Scope;
use rayon_adaptive::atomiclist::AtomicList;
use rayon_adaptive::atomiclist::*;
use rayon_adaptive::fuse_slices;
use rayon_adaptive::prelude::*;
use rayon_adaptive::utils::powers;
use rayon_core::current_thread_index;
use std::cmp::min;
use std::iter::once;
use std::iter::{repeat, repeat_with};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

// const NOTHREAD: ThreadId = ThreadId::max_value();
// type ThreadId = usize;
//
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

// fn slave_spawn<'scope, FOLD2, I, ID2, O2>(
//     retrieving_booleans: &'scope Vec<AtomicBool>,
//     id2: ID2,
//     fold2: FOLD2,
//     scope: &Scope<'scope>,
// ) -> (Sender<Option<Link<O2, I>>>, Arc<AtomicUsize>)
// where
//     I: Divisible + 'scope,
//     ID2: Fn() -> O2 + Sync + Send + Copy + 'scope,
//     O2: Send + Sync + 'scope,
//     FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'scope,
// {
//     let (sender, receiver) = channel::<Option<Link<O2, I>>>();
//     let stolen = Arc::new(AtomicUsize::new(NOTHREAD));
//     let stolen_copy = stolen.clone();
//     scope.spawn(move |same_scope| {
//         stolen_copy.store(current_thread_index().unwrap(), Ordering::SeqCst);
//         let received = receiver.recv().expect("receiving failed");
//         if received.is_none() {
//             return;
//         }
//         let slave_node = received.unwrap();
//         schedule_slave(id2, fold2, slave_node, retrieving_booleans, same_scope);
//     });
//     (sender, stolen)
// }
//
// fn schedule_slave<'scope, I, O2, FOLD2, ID2>(
//     id2: ID2,
//     fold2: FOLD2,
//     node: Link<O2, I>,
//     retrieving_booleans: &'scope Vec<AtomicBool>,
//     scope: &Scope<'scope>,
// ) where
//     I: Divisible + 'scope,
//     ID2: Fn() -> O2 + Sync + Send + Copy + 'scope,
//     O2: Send + Sync + 'scope,
//     FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'scope,
// {
//     let remaining_input = node.take_input();
//     debug_assert!(remaining_input.is_some());
//     let mut remaining_input = remaining_input.unwrap();
//     debug_assert!(remaining_input.base_length() > 0);
//     let mut current_block_size =
//         compute_size(remaining_input.base_length(), default_min_block_size);
//     let mut min_block_size = compute_size(remaining_input.base_length(), default_min_block_size);
//     let mut max_block_size = compute_size(remaining_input.base_length(), default_max_block_size);
//     let mut partial_output = id2();
//     while remaining_input.base_length() > 0 {
//         let (mut sender, mut stolen) = slave_spawn(retrieving_booleans, id2, fold2, scope);
//         loop {
//             if remaining_input.base_length() == 0
//                 || retrieving_booleans[current_thread_index().unwrap()].load(Ordering::SeqCst)
//             {
//                 retrieving_booleans[current_thread_index().unwrap()].store(false, Ordering::SeqCst);
//                 sender.send(None).expect("Steal cancel failed");
//                 if remaining_input.base_length() > 0 {
//                     node.store_input(remaining_input);
//                     node.store_output(partial_output);
//                     return;
//                 }
//                 break;
//             }
//             if stolen.load(Ordering::SeqCst) != NOTHREAD {
//                 if remaining_input.base_length() == 0 {
//                     sender.send(None).expect("Steal cancel failed");
//                     break;
//                 }
//                 if remaining_input.base_length() > min_block_size {
//                     let (my_half, his_half) = remaining_input.divide();
//                     debug_assert!(my_half.base_length() != 0 && his_half.base_length() != 0);
//                     let slave_node = node.split((&stolen).load(Ordering::SeqCst), his_half);
//                     sender.send(Some(slave_node)).expect("Sending work failed!");
//                     remaining_input = my_half;
//                     current_block_size =
//                         compute_size(remaining_input.base_length(), default_min_block_size);
//                     max_block_size =
//                         compute_size(remaining_input.base_length(), default_max_block_size);
//                     min_block_size =
//                         compute_size(remaining_input.base_length(), default_min_block_size);
//                     let rustc_does_not_allow_mutable_tuples_on_the_left_side =
//                         slave_spawn(retrieving_booleans, id2, fold2, scope);
//                     sender = rustc_does_not_allow_mutable_tuples_on_the_left_side.0;
//                     stolen = rustc_does_not_allow_mutable_tuples_on_the_left_side.1;
//                 } else {
//                     sender.send(None).expect("Steal cancel failed");
//                     let remaining_input_length = remaining_input.base_length();
//                     let temp = fold2(partial_output, remaining_input, remaining_input_length);
//                     partial_output = temp.0;
//                     remaining_input = temp.1; //Just to shut up the borrow checker
//                     break;
//                 }
//             } else {
//                 let temp = fold2(partial_output, remaining_input, current_block_size);
//                 partial_output = temp.0;
//                 remaining_input = temp.1;
//                 current_block_size = min(
//                     current_block_size * 2,
//                     min(max_block_size, remaining_input.base_length()),
//                 );
//             }
//         }
//     }
//     node.store_output(partial_output);
// }

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
    input
        .chunks(repeat(macro_block_size))
        .flat_map(|chunk| {
            stolen_stuffs.push_front((None, Some(chunk)));
            stolen_stuffs.iter().flat_map(|(o2, i)| {
                o2.map(|o| FoldElement::Output(o))
                    .into_iter()
                    .chain(i.map(|i| FoldElement::Input(i)).into_iter())
            })
        })
        .fold(o1, |o1, element| match element {
            FoldElement::Input(i) => master_work(o1, i, fold1, stolen_stuffs, nano_block_size),
            FoldElement::Output(o2) => retrieve(o1, o2),
        })
    //     let input_length = i.base_length();
    //     let mut min_block_size = compute_size(input_length, default_min_block_size);
    //     let mut max_block_size = compute_size(input_length, default_max_block_size);
    //     let retrieving_booleans: Vec<_> = repeat_with(|| AtomicBool::new(false))
    //         .take(rayon::current_num_threads())
    //         .collect();
    //     let retrieving_booleans = &retrieving_booleans;
    //     let chunk_size = ((input_length as f64).sqrt().ceil()) as usize;
    //     rayon::scope(|inner_scope| {
    //         i.chunks(repeat(chunk_size))
    //             .fold(o1, |mut partial_output, chunk| {
    //                 let mut current_block_size = min(min_block_size, chunk.base_length());
    //                 let mut linkedlist = LinkedList::new(chunk, retrieve);
    //                 while linkedlist.remaining_input_length() > 0 {
    //                     let (mut sender, mut stolen) =
    //                         slave_spawn(retrieving_booleans, id2, fold2, inner_scope);
    //                     loop {
    //                         if linkedlist.remaining_input_length() == 0 {
    //                             sender.send(None).expect("Steal cancel failed");
    //                             break;
    //                         }
    //                         if stolen.load(Ordering::SeqCst) == NOTHREAD
    //                             || linkedlist.remaining_input_length() <= min_block_size
    //                         {
    //                             assert!(current_block_size <= linkedlist.remaining_input_length());
    //                             // You have input and you were not stolen or stolen too late.
    //                             let temp = fold1(
    //                                 partial_output,
    //                                 linkedlist.take_input().unwrap(),
    //                                 current_block_size,
    //                             );
    //                             partial_output = temp.0;
    //                             if temp.1.base_length() == 0 {
    //                                 sender.send(None).expect("Steal cancel failed");
    //                                 break;
    //                             }
    //                             linkedlist.store_input(temp.1);
    //                             current_block_size = min(
    //                                 current_block_size * 2,
    //                                 min(max_block_size, linkedlist.remaining_input_length()),
    //                             );
    //                         } else {
    //                             // You have input and were stolen.
    //                             let (my_half, his_half) = linkedlist.take_input().unwrap().divide();
    //                             debug_assert!(
    //                                 my_half.base_length() != 0 && his_half.base_length() != 0
    //                             );
    //                             let slave_node =
    //                                 linkedlist.push_node(his_half, stolen.load(Ordering::Relaxed));
    //                             sender.send(Some(slave_node)).expect("Sending work failed!");
    //                             linkedlist.store_input(my_half);
    //                             min_block_size = compute_size(
    //                                 linkedlist.remaining_input_length(),
    //                                 default_min_block_size,
    //                             );
    //                             max_block_size = compute_size(
    //                                 linkedlist.remaining_input_length(),
    //                                 default_max_block_size,
    //                             );
    //                             current_block_size =
    //                                 min(min_block_size, linkedlist.remaining_input_length());
    //                             let temp = slave_spawn(retrieving_booleans, id2, fold2, inner_scope);
    //                             sender = temp.0;
    //                             stolen = temp.1;
    //                         }
    //                     }
    //                     let retrieve_result =
    //                         linkedlist.start_retrieve(partial_output, retrieving_booleans);
    //                     partial_output = retrieve_result.0;
    //                     linkedlist = retrieve_result.1;
    //                     current_block_size = min(min_block_size, linkedlist.remaining_input_length());
    //                     assert!(current_block_size <= linkedlist.remaining_input_length());
    //                 }
    //                 partial_output
    //             })
    //     })
}

fn master_work<I, O1, O2, FOLD1>(
    init: O1,
    input: I,
    fold: FOLD1,
    stolen_stuffs: &AtomicList<(Option<O2>, Option<I>)>,
    initial_size: usize,
) -> O1
where
    I: DivisibleIntoBlocks,
    O1: Send + Sync + Copy,
    O2: Send + Sync,
    FOLD1: Fn(O1, I, usize) -> (O1, I) + Sync + Send + Copy,
{
    let stolen = &AtomicBool::new(false);
    // let's work sequentially until stolen
    match powers(initial_size)
        .take_while(|_| !stolen.load(Ordering::Relaxed))
        .try_fold((init, input), |(output, input), size| {
            let checked_size = min(input.base_length(), size);
            if checked_size > 0 {
                Ok(fold(output, input, checked_size))
            } else {
                Err(output)
            }
        }) {
        Ok((output, input)) => panic!("TODO: we were stolen!"),
        Err(output) => panic!("everything done!"),
    }
}

fn main() {
    (2..3).for_each(|number_of_threads| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build()
            .expect("Thread pool build failed");
        let mut input_vector = vec![1; 100_000];
        let time_taken_ms = pool.scope(|s| {
            let start = time::precise_time_ns();
            fold_with_help(
                input_vector.as_mut_slice(),
                0,
                |last_elem_prev_slice, remaining_slice, limit| {
                    let (todo, remaining) = remaining_slice.divide_at(limit);
                    (
                        todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                            *e = *e + c;
                            e.clone()
                        }),
                        remaining,
                    )
                },
                || None,
                |possible_previous_slice: Option<&mut [u32]>, input, limit| {
                    let last_elem_prev_slice = possible_previous_slice
                        .as_ref()
                        .and_then(|c| c.last().cloned())
                        .unwrap_or(0);
                    let (todo, remaining) = input.divide_at(limit);
                    todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                        *e = *e + c;
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
                                .for_each(|e| *e += last_num)
                        });
                        last_num + last_slice_num
                    } else {
                        last_num
                    }
                },
            );
            let end = time::precise_time_ns();
            ((end - start) as f64) / (1e6 as f64)
        });
        let expected_result: Vec<_> = (1..100_001).into_iter().collect();
        assert_eq!(input_vector, expected_result);

        println!("{}, {}", time_taken_ms, number_of_threads);
    });
}
