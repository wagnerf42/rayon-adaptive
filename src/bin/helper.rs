use atomic_option::AtomicOption;
use rayon::current_num_threads;
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
type Link<O2, I> = Arc<Node<O2, I>>;

struct Node<O2, I> {
    id: ThreadId,
    finished: AtomicOption<bool>,

    output: AtomicOption<O2>,
    input: AtomicOption<I>,
    next: AtomicOption<Link<O2, I>>,
}

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
fn split<O2, I>(split_me: Link<O2, I>, id: ThreadId) -> Link<O2, I> {
    let new_node = Arc::new(Node {
        id,
        finished: AtomicOption::empty(),
        output: AtomicOption::empty(),
        input: AtomicOption::empty(),
        next: AtomicOption::empty(),
    });
    let new_node_clone = new_node.clone();
    new_node
        .next
        .replace(split_me.next.take(Ordering::SeqCst), Ordering::SeqCst);
    split_me
        .next
        .replace(Some(Box::new(new_node)), Ordering::SeqCst);
    new_node_clone
}

fn slave_spawn<FOLD2, I, ID2, O2>(
    vector: &Arc<Vec<AtomicBool>>,
    id2: ID2,
    fold2: FOLD2,
) -> (Sender<Option<(I, Link<O2, I>)>>, Arc<AtomicUsize>)
where
    I: DivisibleIntoBlocks + 'static,
    ID2: Fn() -> O2 + Sync + Send + Copy + 'static,
    O2: Send + Sync + 'static,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync + Send + Copy + 'static,
{
    let (sender, receiver) = channel::<Option<(I, Link<O2, I>)>>();
    let stolen = Arc::new(AtomicUsize::new(NOTHREAD));
    let vector_clone = vector.clone();
    let stolen_copy = stolen.clone();
    rayon::spawn(move || {
        stolen_copy.store(current_thread_index().unwrap(), Ordering::SeqCst);
        let mut input = receiver.recv().expect("receiving failed");
        let input = input.take();
        if input.is_none() {
            return;
        } else {
            let (work, slave_node) = input.unwrap();
            assert!(work.base_length() > 0);
            schedule_slave(work, id2, fold2, slave_node, vector_clone);
        }
    });
    (sender, stolen)
}

fn schedule_slave<I, O2, FOLD2, ID2>(
    i: I,
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
    let mut remaining_input = i;
    let mut current_block_size =
        compute_size(remaining_input.base_length(), default_min_block_size);
    let mut max_block_size = compute_size(remaining_input.base_length(), default_max_block_size);
    let mut partial_output = id2();
    //let mut stolen: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(NOTHREAD));
    while remaining_input.base_length() > 0 {
        let (mut sender, mut stolen) = slave_spawn(&vector, id2, fold2);
        loop {
            if remaining_input.base_length() == 0
                || vector[current_thread_index().unwrap()].load(Ordering::SeqCst) == true
            {
                vector[current_thread_index().unwrap()].store(false, Ordering::SeqCst);
                sender.send(None).expect("Steal cancel failed");
                if remaining_input.base_length() > 0 {
                    node.input.swap(Box::new(remaining_input), Ordering::SeqCst);
                }
                node.output.swap(Box::new(partial_output), Ordering::SeqCst);
                node.finished.swap(Box::new(true), Ordering::SeqCst);
                return;
            }
            let temp = fold2(partial_output, remaining_input, current_block_size);
            partial_output = temp.0;
            remaining_input = temp.1;
            if stolen.load(Ordering::SeqCst) != NOTHREAD {
                if remaining_input.base_length() == 0 {
                    sender.send(None).expect("Steal cancel failed");
                    break;
                }
                let (my_half, his_half) = remaining_input.divide();
                sender
                    .send(Some((
                        his_half,
                        split((&node).clone(), (&stolen).load(Ordering::SeqCst)),
                    )))
                    .expect("Sending work failed!");
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
            current_block_size = min(
                current_block_size * 2,
                min(max_block_size, remaining_input.base_length()),
            );
        }
    }
    node.finished.swap(Box::new(true), Ordering::SeqCst);
}

// Consume the list only as long as you have only O2s in the nodes. As soon as you encounter O2,
// I(non-empty), return a head node along with the O1 that you (may) have generated in this function.
// The head NEVER contains an input!
fn start_retrieve<O2, I, RET, O1>(
    processed_output: O1,
    head: Link<O2, I>,
    retrieve_fn: RET,
    vector: Arc<Vec<AtomicBool>>,
) -> (O1, Link<O2, I>)
where
    I: DivisibleIntoBlocks,
    O2: Send + Sync + Sized,
    RET: Fn(O1, O2) -> O1 + Sync,
    O1: Send + Sync,
{
    let mut iter_link = Some(Box::new(head));
    let mut partial_output = processed_output;
    let return_node = Arc::new(Node {
        id: current_thread_index().unwrap(),
        finished: AtomicOption::empty(),
        output: AtomicOption::empty(),
        input: AtomicOption::empty(),
        next: AtomicOption::empty(),
    });
    while iter_link.is_some() {
        let task_node = iter_link.unwrap();
        if task_node.id != current_thread_index().unwrap() {
            //Pessimistically signal it and spinlock on the option.
            vector[task_node.id].store(true, Ordering::SeqCst);
            task_node.finished.spinlock(Ordering::SeqCst);
            let his_output = task_node.output.take(Ordering::SeqCst);
            let his_input = task_node.input.take(Ordering::SeqCst);
            if his_output.is_some() {
                partial_output = retrieve_fn(partial_output, *his_output.unwrap());
            }
            if his_input.is_some() {
                //let his_input = his_input.unwrap();
                assert!(his_input.as_ref().unwrap().base_length() > 0);
                return_node.input.replace(his_input, Ordering::SeqCst);
                return_node
                    .next
                    .replace(task_node.next.take(Ordering::SeqCst), Ordering::SeqCst);
                break;
            }
        }
        iter_link = task_node.next.take(Ordering::SeqCst);
    }
    return (partial_output, return_node);
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
    let mut head: Link<O2, I> = Arc::new(Node {
        id: current_thread_index().unwrap(),
        finished: AtomicOption::empty(),
        output: AtomicOption::empty(),
        input: AtomicOption::empty(),
        next: AtomicOption::empty(),
    });
    let mut remaining_input = i;
    let mut partial_output = o1;
    if remaining_input.base_length() <= MINSIZE {
        let length = remaining_input.base_length();
        return fold1(partial_output, remaining_input, length).0;
    }
    let mut current_block_size =
        compute_size(remaining_input.base_length(), default_min_block_size);
    let mut max_block_size = compute_size(remaining_input.base_length(), default_max_block_size);
    let retrieve_vec: Vec<_> = repeat_with(|| AtomicBool::new(false))
        .take(rayon::current_num_threads())
        .collect();
    let retrieve_vec = Arc::new(retrieve_vec);
    while remaining_input.base_length() > 0 {
        let (mut sender, mut stolen) = slave_spawn(&retrieve_vec, id2, fold2);
        loop {
            if remaining_input.base_length() == 0 {
                sender.send(None).expect("Steal cancel failed");
                break;
            }
            let temp = fold1(partial_output, remaining_input, current_block_size);
            partial_output = temp.0;
            remaining_input = temp.1;
            if stolen.load(Ordering::SeqCst) != NOTHREAD {
                if remaining_input.base_length() == 0 {
                    sender.send(None).expect("Steal cancel failed");
                    break;
                }
                let (my_half, his_half) = remaining_input.divide();
                sender
                    .send(Some((
                        his_half,
                        split((&head).clone(), (&stolen).load(Ordering::SeqCst)),
                    )))
                    .expect("Sending work failed!");
                remaining_input = my_half;
                max_block_size =
                    compute_size(remaining_input.base_length(), default_max_block_size);
                current_block_size =
                    compute_size(remaining_input.base_length(), default_min_block_size);
                let rustc_does_not_allow_mutable_tuples_on_the_left_side =
                    slave_spawn(&retrieve_vec, id2, fold2);
                sender = rustc_does_not_allow_mutable_tuples_on_the_left_side.0;
                stolen = rustc_does_not_allow_mutable_tuples_on_the_left_side.1;
                continue;
            }
            current_block_size = min(
                current_block_size * 2,
                min(max_block_size, remaining_input.base_length()),
            );
        }
        let res = start_retrieve(partial_output, head, retrieve, retrieve_vec.clone());
        partial_output = res.0;
        head = res.1;
        let maybe_input = head.input.take(Ordering::SeqCst);
        match maybe_input {
            Some(input) => {
                remaining_input = *input;
            }
            None => {
                break;
            }
        }
    }
    partial_output
}

fn main() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(10)
        .build()
        .expect("Thread pool build failed");
    pool.install(|| {
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
                let len_before_divide = i.base_length();
                let (todo, remaining) = i.divide_at(limit);
                let left_len = todo.base_length();
                let right_len = remaining.base_length();
                let len_after_divide = right_len + left_len;
                assert_eq!(len_before_divide, len_after_divide);
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
    });
}
