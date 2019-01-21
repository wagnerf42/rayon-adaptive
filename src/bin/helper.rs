use crossbeam::atomic::AtomicCell;
use rayon::current_num_threads;
use rayon_adaptive::linkedlist::*;
use rayon_adaptive::prelude::*;
use rayon_core::current_thread_index;
use std::cmp::min;
use std::iter::repeat_with;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
const MINSIZE: usize = 10;
const NOTHREAD: ThreadId = ThreadId::max_value();
type ThreadId = usize;

fn f(e: usize) -> usize {
    let mut c = 0;
    for x in 0..e {
        c += x;
    }
    c
}

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

fn slave_spawn<FOLD2, I, ID2, O2>(
    vector: &Arc<Vec<AtomicBool>>,
    id2: ID2,
    fold2: FOLD2,
) -> (Sender<Option<Link<O2, I>>>, Arc<AtomicUsize>)
where
    I: DivisibleIntoBlocks + 'static,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'static,
    O2: Send + Sync + 'static,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'static,
{
    let (sender, receiver) = channel::<Option<Link<O2, I>>>();
    let stolen = Arc::new(AtomicUsize::new(NOTHREAD));
    let vector_clone = vector.clone();
    let stolen_copy = stolen.clone();
    rayon::spawn(move || {
        stolen_copy.store(current_thread_index().unwrap(), Ordering::SeqCst);
        let mut received = receiver.recv().expect("receiving failed");
        if received.is_none() {
            return;
        }
        let slave_node = received.unwrap();
        schedule_slave(id2, fold2, slave_node, vector_clone);
    });
    (sender, stolen)
}

fn schedule_slave<I, O2, FOLD2, ID2>(
    id2: ID2,
    fold2: FOLD2,
    node: Link<O2, I>,
    vector: Arc<Vec<AtomicBool>>,
) where
    I: DivisibleIntoBlocks + 'static,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'static,
    O2: Send + Sync + 'static,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'static,
{
    let remaining_input = node.take_input();
    debug_assert!(remaining_input.is_some());
    let mut remaining_input = remaining_input.unwrap();
    debug_assert!(remaining_input.base_length() > 0);
    let mut current_block_size =
        compute_size(remaining_input.base_length(), default_min_block_size);
    let mut max_block_size = compute_size(remaining_input.base_length(), default_max_block_size);
    let mut partial_output = id2();
    while remaining_input.base_length() > 0 {
        let (mut sender, mut stolen) = slave_spawn(&vector, id2, fold2);
        loop {
            if remaining_input.base_length() == 0
                || vector[current_thread_index().unwrap()].load(Ordering::SeqCst) == true
            {
                vector[current_thread_index().unwrap()].store(false, Ordering::SeqCst);
                sender.send(None).expect("Steal cancel failed");
                if remaining_input.base_length() > 0 {
                    node.store_input(remaining_input);
                    node.store_output(partial_output);
                    return; //bad design, but borrow checker complains if I break.
                }
                break;
            }
            if stolen.load(Ordering::SeqCst) != NOTHREAD {
                if remaining_input.base_length() == 0 {
                    sender.send(None).expect("Steal cancel failed");
                    break;
                }
                let (my_half, his_half) = remaining_input.divide();
                if his_half.base_length() == 0 {
                    sender.send(None).expect("Steal cancel failed");
                } else {
                    let slave_node = node.split((&stolen).load(Ordering::SeqCst), his_half);
                    sender.send(Some(slave_node)).expect("Sending work failed!");
                }
                remaining_input = my_half;
                current_block_size =
                    compute_size(remaining_input.base_length(), default_min_block_size);
                max_block_size =
                    compute_size(remaining_input.base_length(), default_max_block_size);
                let rustc_does_not_allow_mutable_tuples_on_the_left_side =
                    slave_spawn(&vector, id2, fold2);
                sender = rustc_does_not_allow_mutable_tuples_on_the_left_side.0;
                stolen = rustc_does_not_allow_mutable_tuples_on_the_left_side.1;
                continue;
            }
            let temp = fold2(partial_output, remaining_input, current_block_size);
            partial_output = temp.0;
            remaining_input = temp.1;
            current_block_size = min(
                current_block_size * 2,
                min(max_block_size, remaining_input.base_length()),
            );
        }
    }
    node.store_output(partial_output);
}

fn fold_with_help<I, O1, O2, ID2, FOLD1, FOLD2, RET>(
    i: I,
    o1: O1,
    fold1: FOLD1,
    id2: ID2,
    fold2: FOLD2,
    retrieve: RET,
) -> O1
where
    I: DivisibleAtIndex + 'static,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'static,
    O1: Send + Sync,
    O2: Send + Sync + 'static,
    FOLD1: Fn(O1, I, usize) -> (O1, I) + Sync + Send + Copy,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'static,
    RET: Fn(O1, O2) -> O1 + Sync + Copy,
{
    let mut linkedlist = LinkedList::new(i, retrieve);
    let mut partial_output = o1;
    let remaining_input_length = linkedlist.remaining_input_length();
    if remaining_input_length <= MINSIZE {
        return fold1(
            partial_output,
            linkedlist.take_input().unwrap(),
            remaining_input_length,
        )
        .0;
    }
    let mut current_block_size = compute_size(remaining_input_length, default_min_block_size);
    let mut max_block_size = compute_size(remaining_input_length, default_max_block_size);
    let retrieve_vec: Vec<_> = repeat_with(|| AtomicBool::new(false))
        .take(rayon::current_num_threads())
        .collect();
    let retrieve_vec = Arc::new(retrieve_vec);
    while linkedlist.remaining_input_length() > 0 {
        let (mut sender, mut stolen) = slave_spawn(&retrieve_vec, id2, fold2);
        loop {
            if linkedlist.remaining_input_length() == 0 {
                sender.send(None).expect("Steal cancel failed");
                break;
            }
            if stolen.load(Ordering::SeqCst) == NOTHREAD {
                let temp = fold1(
                    partial_output,
                    linkedlist.take_input().unwrap(),
                    current_block_size,
                );
                partial_output = temp.0;
                if temp.1.base_length() == 0 {
                    sender.send(None).expect("Steal cancel failed");
                    break;
                }
                linkedlist.store_input(temp.1);
                current_block_size = min(
                    current_block_size * 2,
                    min(max_block_size, linkedlist.remaining_input_length()),
                );
            } else {
                let (my_half, his_half) = linkedlist.take_input().unwrap().divide();
                debug_assert!(my_half.base_length() != 0 || his_half.base_length() != 0);
                if his_half.base_length() != 0 {
                    let slave_node = linkedlist.push_node(his_half, stolen.load(Ordering::Relaxed));
                    sender.send(Some(slave_node)).expect("Sending work failed!");
                    if my_half.base_length() != 0 {
                        linkedlist.store_input(my_half);
                        max_block_size = compute_size(
                            linkedlist.remaining_input_length(),
                            default_max_block_size,
                        );
                        current_block_size = compute_size(
                            linkedlist.remaining_input_length(),
                            default_min_block_size,
                        );
                        let rustc_does_not_allow_mutable_tuples_on_the_left_side =
                            slave_spawn(&retrieve_vec, id2, fold2);
                        sender = rustc_does_not_allow_mutable_tuples_on_the_left_side.0;
                        stolen = rustc_does_not_allow_mutable_tuples_on_the_left_side.1;
                    } else {
                        break;
                    }
                } else {
                    sender.send(None).expect("Steal cancel failed");
                    let temp_len = my_half.base_length();
                    partial_output = fold1(partial_output, my_half, temp_len).0;
                    break;
                }
            }
        }
        let retrieve_result = linkedlist.start_retrieve(partial_output, retrieve_vec.clone());
        partial_output = retrieve_result.0;
        linkedlist = retrieve_result.1;
    }
    partial_output
}

fn main() {
    (1..10).for_each(|number_of_threads| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build()
            .expect("Thread pool build failed");
        let time_taken_ms = pool.install(|| {
            let start = time::precise_time_ns();
            fold_with_help(
                0..100_000,
                (),
                |_, i, limit| {
                    let (todo, remaining) = i.divide_at(limit);
                    for e in todo {
                        println!("{}", f(e));
                    }
                    ((), remaining)
                },
                Vec::new,
                |mut v, i, limit| {
                    let (todo, remaining) = i.divide_at(limit);
                    v.extend(todo.into_iter().map(|e| f(e)));
                    (v, remaining)
                },
                |_, v| {
                    for e in &v {
                        println!("{}", e);
                    }
                    ()
                },
            );
            let end = time::precise_time_ns();
            ((end - start) as f64) / (1e6 as f64)
        });
        println!("{}, {}", time_taken_ms, number_of_threads);
    });
}
