//! Fully adaptive algorithms with work.
//! see the adaptive_prefix_work example.

// Note that there is a lot of copy paste in this file from help.rs
// genericity is HARD and time consuming.
// so i'll just start with something that works and later on when time is available
// try to re-factor things.

// TODO: we could do with a list of inputs instead of a list of couples

use crate::atomiclist::{AtomicLink, AtomicList};
use crate::prelude::*;
use crate::small_channel::{small_channel, SmallSender};
use crate::utils::power_sizes;
use rayon::Scope;
use std::cmp::min;
use std::iter::once;
use std::mem;

/// Remember how to work helping sequential thread.
pub struct HelpWork<I, H> {
    pub(crate) input: I,
    pub(crate) help_op: H,
    pub(crate) sizes: Box<Iterator<Item = usize> + Send>,
}

impl<I: Divisible, H: Fn(I, usize) -> I> HelpWork<I, H> {
    /// Set the macro-blocks sizes.
    pub fn by_blocks<S>(mut self, sizes: S) -> Self
    where
        S: Iterator<Item = usize> + 'static + Send,
    {
        self.sizes = Box::new(sizes);
        self
    }
}

// TODO will the IntoIterator here collision with the one from the ParallelIterator trait ?
impl<I: Divisible + IntoIterator + Send, H: Fn(I, usize) -> I + Sync> HelpWork<I, H> {
    /// Fold sequentially with help.
    pub fn fold<B, F, R>(self, initial_value: B, fold_op: F, retrieve_op: R) -> B
    where
        B: Send,
        F: Fn(B, I::Item) -> B + Sync,
        R: Fn(B, I) -> B + Sync,
    {
        let stolen_stuffs: &AtomicList<I> = &AtomicList::new();
        let sizes = self.sizes;
        let input = self.input;
        let help_op = self.help_op;
        // TODO: the sizes unwraps here don't make sense.
        // wouldn't it be better to compute the sizes inside each macro-block ?
        let min_block_size = input
            .base_length()
            .map(|s| (s as f64).log2() as usize)
            .unwrap_or(1);
        let max_block_size = input
            .base_length()
            .map(|s| (s as f64).sqrt() as usize)
            .unwrap_or(100_000);
        rayon::scope(|s| {
            input
                .blocks(sizes)
                .flat_map(|block| once(block).chain(stolen_stuffs.iter()))
                .fold(initial_value, |b, i| {
                    // now, this division is crucial.
                    // by dividing at 0 we separate the computed result in done
                    // and the remaining input in remaining
                    // we can then retrieve the 'done' part and let the sequential thread
                    // go on with the remaining part.
                    let (done, remaining) = i.divide_at(0);
                    StealAnswerer::new(
                        s,
                        remaining,
                        &help_op,
                        stolen_stuffs,
                        (min_block_size, max_block_size),
                    )
                    .flatten()
                    .fold(retrieve_op(b, done), &fold_op)
                })
        })
    }
}

/// This structure is used by the sequential worker
/// to answer steal requests.
/// It acts as an iterator on all sequential iterators produced between steal requests.
struct StealAnswerer<'a, 'c, 'scope, I, H> {
    scope: &'a Scope<'scope>,
    sizes_bounds: (usize, usize),
    sizes: Box<Iterator<Item = usize>>,
    input: Option<I>,
    help_op: &'scope H,
    stolen_stuffs: &'c AtomicList<I>,
    sender: SmallSender<AtomicLink<I>>,
}

impl<'a, 'c, 'scope, I, H> StealAnswerer<'a, 'c, 'scope, I, H>
where
    I: Divisible + Send + 'scope,
    H: Fn(I, usize) -> I + Sync,
{
    fn new(
        scope: &'a Scope<'scope>,
        input: I,
        help_op: &'scope H,
        stolen_stuffs: &'c AtomicList<I>,
        sizes_bounds: (usize, usize),
    ) -> Self {
        let sender = spawn_stealing_task(scope, help_op, sizes_bounds);
        StealAnswerer {
            scope,
            sizes_bounds,
            sizes: Box::new(power_sizes(sizes_bounds.0, sizes_bounds.1)),
            input: Some(input),
            help_op,
            stolen_stuffs,
            sender,
        }
    }
}

impl<'a, 'c, 'scope, I, H> Iterator for StealAnswerer<'a, 'c, 'scope, I, H>
where
    I: Divisible + IntoIterator + Send + 'scope,
    H: Fn(I, usize) -> I + Sync + 'scope,
{
    type Item = <I as IntoIterator>::IntoIter;
    fn next(&mut self) -> Option<Self::Item> {
        let mut input = self.input.take().unwrap();
        let remaining_length = input.base_length().expect("infinite input");
        if remaining_length == 0 {
            None
        } else {
            if self.sender.receiver_is_waiting() && remaining_length > self.sizes_bounds.0 {
                // let's split, we have enough for both
                let (my_half, his_half) = input.divide();
                if his_half.base_length().expect("infinite input") > 0 {
                    // TODO: remove this if ?
                    let stolen_node = self.stolen_stuffs.push_front(his_half);
                    let mut new_sender =
                        spawn_stealing_task(self.scope, self.help_op, self.sizes_bounds);
                    mem::swap(&mut new_sender, &mut self.sender);
                    new_sender.send(stolen_node);
                }
                input = my_half;
            }
            let next_length = self.sizes.next().unwrap();
            let checked_length = min(next_length, input.base_length().expect("infinite input"));
            let (first_part, remaining_part) = input.divide_at(checked_length);
            self.input = Some(remaining_part);
            Some(first_part.into_iter())
        }
    }
}

fn spawn_stealing_task<'scope, I, H>(
    scope: &Scope<'scope>,
    help_op: &'scope H,
    sizes_bounds: (usize, usize),
) -> SmallSender<AtomicLink<I>>
where
    I: Divisible + Send + 'scope,
    H: Fn(I, usize) -> I + Sync,
{
    let (sender, receiver) = small_channel();
    scope.spawn(move |s| {
        let stolen_input: Option<AtomicLink<I>>;
        #[cfg(feature = "logs")]
        {
            stolen_input = rayon_logs::subgraph("slave wait", 1, || receiver.recv());
        }
        #[cfg(not(feature = "logs"))]
        {
            stolen_input = receiver.recv();
        }
        if let Some(node) = stolen_input {
            helper_work(s, node, help_op, sizes_bounds)
        }
    });
    sender
}

fn helper_work<'scope, I, H>(
    scope: &Scope<'scope>,
    node: AtomicLink<I>,
    help_op: &'scope H,
    sizes_bounds: (usize, usize),
) where
    I: Divisible + Send + 'scope,
    H: Fn(I, usize) -> I + Sync + 'scope,
{
    let mut input = node.take().unwrap();
    loop {
        let sender = spawn_stealing_task(scope, help_op, sizes_bounds);
        match power_sizes(sizes_bounds.0, sizes_bounds.1)
            .take_while(|_| !sender.receiver_is_waiting() && !node.requested())
            .try_fold(input, |input, size| {
                let remaining_size = input.base_length().expect("infinite iterator");
                if remaining_size > 0 {
                    Ok((help_op)(input, size))
                } else {
                    Err(input)
                }
            }) {
            Ok(remaining_input) => {
                // we were stolen or retrieved
                if node.requested() {
                    // we were retrieved
                    node.replace(remaining_input);
                    return;
                } else {
                    // we were stolen
                    let length = remaining_input.base_length().expect("infinite iterator");
                    if length > sizes_bounds.0 {
                        let (my_half, his_half) = remaining_input.divide();
                        if his_half.base_length().expect("infinite iterator") > 0 {
                            let stolen_node = (&node).split(his_half);
                            sender.send(stolen_node)
                        }
                        input = my_half;
                    } else {
                        input = remaining_input;
                    }
                }
            }
            Err(final_input) => {
                node.replace(final_input);
                return;
            }
        }
    }
}
