//! Fully adaptive algorithms with work.
//! see the adaptive_prefix_work example.

// Note that there is a lot of copy paste in this file from help.rs
// genericity is HARD and time consuming.
// so i'll just start with something that works and later on when time is available
// try to re-factor things.

// TODO: we could do with a list of inputs instead of a list of couples
// TODO: we need to put block sizes in here

use crate::atomiclist::{AtomicLink, AtomicList};
use crate::help::RemainingElement;
use crate::prelude::*;
use crate::small_channel::{small_channel, SmallSender};
use crate::utils::power_sizes;
use rayon::Scope;
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
        let stolen_stuffs: &AtomicList<(Option<I>, Option<I>)> = &AtomicList::new();
        let sizes = self.sizes;
        let input = self.input;
        let help_op = self.help_op;
        rayon::scope(|s| {
            input
                .blocks(sizes)
                .flat_map(|block| {
                    once(RemainingElement::Input(block)).chain(stolen_stuffs.iter().flat_map(
                        |(c, i)| {
                            c.map(RemainingElement::Output)
                                .into_iter()
                                .chain(i.map(RemainingElement::Input).into_iter())
                        },
                    ))
                })
                .fold(initial_value, |b, element| match element {
                    RemainingElement::Input(i) => StealAnswerer::new(s, i, &help_op, stolen_stuffs)
                        .flatten()
                        .fold(b, &fold_op),
                    RemainingElement::Output(i) => retrieve_op(b, i),
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
    stolen_stuffs: &'c AtomicList<(Option<I>, Option<I>)>,
    sender: SmallSender<AtomicLink<(Option<I>, Option<I>)>>,
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
        stolen_stuffs: &'c AtomicList<(Option<I>, Option<I>)>,
    ) -> Self {
        let sender = spawn_stealing_task(scope, help_op);
        StealAnswerer {
            scope,
            sizes_bounds: (100, 10_000), // for now,
            sizes: Box::new(power_sizes(100, 10_000)),
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
                    let stolen_node = self.stolen_stuffs.push_front((None, Some(his_half)));
                    let mut new_sender = spawn_stealing_task(self.scope, self.help_op);
                    mem::swap(&mut new_sender, &mut self.sender);
                    new_sender.send(stolen_node);
                }
                input = my_half;
            }
            let next_length = self.sizes.next().unwrap();
            let (first_part, remaining_part) = input.divide_at(next_length);
            self.input = Some(remaining_part);
            Some(first_part.into_iter())
        }
    }
}

fn spawn_stealing_task<'scope, I, H>(
    scope: &Scope<'scope>,
    help_op: &'scope H,
) -> SmallSender<AtomicLink<(Option<I>, Option<I>)>>
where
    I: Divisible + Send + 'scope,
    H: Fn(I, usize) -> I + Sync,
{
    let (sender, receiver) = small_channel();
    scope.spawn(move |s| {
        let stolen_input: Option<AtomicLink<(Option<I>, Option<I>)>>;
        #[cfg(feature = "logs")]
        {
            stolen_input = rayon_logs::subgraph("slave wait", 1, || receiver.recv());
        }
        #[cfg(not(feature = "logs"))]
        {
            stolen_input = receiver.recv();
        }
        if let Some(node) = stolen_input {
            helper_work(s, node, help_op)
        }
    });
    sender
}

fn helper_work<'scope, I, H>(
    scope: &Scope<'scope>,
    node: AtomicLink<(Option<I>, Option<I>)>,
    help_op: &'scope H,
) where
    I: Divisible + Send + 'scope,
    H: Fn(I, usize) -> I + Sync + 'scope,
{
    let mut input = node.take().unwrap().1.unwrap();
    loop {
        let sender = spawn_stealing_task(scope, help_op);
        match power_sizes(100, 10_000)
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
                    node.replace(
                        if remaining_input.base_length().expect("infinite iterator") == 0 {
                            (Some(remaining_input), None)
                        } else {
                            (None, Some(remaining_input))
                        },
                    );
                    return;
                } else {
                    // we were stolen
                    let length = remaining_input.base_length().expect("infinite iterator");
                    if length > 100 {
                        let (my_half, his_half) = remaining_input.divide();
                        if his_half.base_length().expect("infinite iterator") > 0 {
                            let stolen_node = (&node).split((None, Some(his_half)));
                            sender.send(stolen_node)
                        }
                        input = my_half;
                    } else {
                        input = remaining_input;
                    }
                }
            }
            Err(final_input) => {
                node.replace((Some(final_input), None));
                return;
            }
        }
    }
}
