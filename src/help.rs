//! This file contains the most complex scheduling algorithm
//! where one thread works sequentially with no overhead and
//! other threads help him (with overhead).
//! The idea is that if no steal occurs, we end up with the sequential algorithm.
use crate::atomiclist::{AtomicLink, AtomicList};
use crate::prelude::*;
use crate::small_channel::{small_channel, SmallSender};
use crate::utils::power_sizes;
use crate::Policy;
use rayon::Scope;
use std::iter;
use std::iter::once;
use std::marker::PhantomData;
use std::mem;

//TODO: do we really need all these lifetimes ?
/// This structure is used by the sequential worker
/// to answer steal requests.
/// It acts as an iterator on all sequential iterators produced between steal requests.
struct StealAnswerer<'a, 'c, 'scope, I, C> {
    scope: &'a Scope<'scope>,
    sizes_bounds: (usize, usize),
    sizes: Box<Iterator<Item = usize>>, // TODO: remove box
    iterator: Option<I>,                // TODO: keep or remove option ? (and be unsafe)
    help_op: &'scope Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
    stolen_stuffs: &'c AtomicList<(Option<C>, Option<I>)>,
    sender: SmallSender<AtomicLink<(Option<C>, Option<I>)>>,
}

impl<'a, 'c, 'scope, I, C> StealAnswerer<'a, 'c, 'scope, I, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
{
    fn new(
        scope: &'a Scope<'scope>,
        iterator: I,
        help_op: &'scope Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
        stolen_stuffs: &'c AtomicList<(Option<C>, Option<I>)>,
    ) -> Self {
        let policy = iterator.policy();
        let sizes_bounds = match policy {
            Policy::Adaptive(min_size, max_size) => (min_size, max_size),
            _ => panic!("only adaptive policies are supported in helper schemes"),
        };
        let sender = spawn_stealing_task(scope, help_op);
        StealAnswerer {
            scope,
            sizes_bounds,
            sizes: Box::new(power_sizes(sizes_bounds.0, sizes_bounds.1)),
            iterator: Some(iterator),
            help_op,
            stolen_stuffs,
            sender,
        }
    }
}

impl<'a, 'c, 'scope, I, C> Iterator for StealAnswerer<'a, 'c, 'scope, I, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
{
    type Item = I::SequentialIterator;
    fn next(&mut self) -> Option<Self::Item> {
        let mut iterator = self.iterator.take().unwrap();
        let remaining_length = iterator.base_length().expect("infinite iterator");
        if remaining_length == 0 {
            None
        } else {
            if self.sender.receiver_is_waiting() && remaining_length > self.sizes_bounds.0 {
                // let's split, we have enough for both
                let (my_half, his_half) = iterator.divide();
                if his_half.base_length().expect("infinite iterator") > 0 {
                    // TODO: remove this if ?
                    let stolen_node = self.stolen_stuffs.push_front((None, Some(his_half)));
                    let mut new_sender = spawn_stealing_task(self.scope, self.help_op);
                    mem::swap(&mut new_sender, &mut self.sender);
                    new_sender.send(stolen_node);
                }
                iterator = my_half;
            }
            let next_length = self.sizes.next().unwrap();
            let (sequential_iterator, remaining_parallel_iterator) = iterator.iter(next_length);
            self.iterator = Some(remaining_parallel_iterator);
            Some(sequential_iterator)
        }
    }
}

/// This structure is used by the helper threads
/// to answer steal requests and retrieve requests.
/// It acts as an iterator on all sequential iterators produced between requests.
pub struct Retriever<'a, 'b, 'scope, I, C> {
    scope: &'a Scope<'scope>,
    sizes_bounds: (usize, usize),
    sizes: Box<Iterator<Item = usize>>, // TODO: remove box
    help_op: &'scope Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
    node: &'b AtomicLink<(Option<C>, Option<I>)>,
    iterator: Option<I>,
    sender: SmallSender<AtomicLink<(Option<C>, Option<I>)>>,
}

impl<'a, 'b, 'scope, I, C> Retriever<'a, 'b, 'scope, I, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
{
    fn new(
        scope: &'a Scope<'scope>,
        help_op: &'scope Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
        node: &'b AtomicLink<(Option<C>, Option<I>)>,
    ) -> Self {
        let iterator = node.take().unwrap().1.unwrap();
        let policy = iterator.policy();
        let sizes_bounds = match policy {
            Policy::Adaptive(min_size, max_size) => (min_size, max_size),
            _ => panic!("only adaptive policies are supported in helper schemes"),
        };
        let sender = spawn_stealing_task(scope, help_op);
        Retriever {
            scope,
            sizes_bounds,
            sizes: Box::new(power_sizes(sizes_bounds.0, sizes_bounds.1)),
            help_op,
            node,
            iterator: Some(iterator),
            sender,
        }
    }
}

impl<'a, 'b, 'scope, I, C> Iterator for Retriever<'a, 'b, 'scope, I, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
{
    type Item = I::SequentialIterator;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iterator.is_none() {
            return None;
        }
        let mut iterator = self.iterator.take().unwrap();
        let remaining_length = iterator.base_length().expect("infinite iterator");
        if remaining_length == 0 {
            None
        } else {
            if self.node.requested() {
                // we are being retrieved.
                // give the iterator back to sequential thread through the list.
                self.node.replace((None, Some(iterator)));
                return None;
            } else if self.sender.receiver_is_waiting() && remaining_length > self.sizes_bounds.0 {
                // let's split, we have enough for both
                let (my_half, his_half) = iterator.divide();
                if his_half.base_length().expect("infinite iterator") > 0 {
                    // TODO: remove this if ?
                    let stolen_node = self.node.split((None, Some(his_half)));
                    let mut new_sender = spawn_stealing_task(self.scope, self.help_op);
                    mem::swap(&mut new_sender, &mut self.sender);
                    new_sender.send(stolen_node);
                }
                iterator = my_half;
            }
            let next_length = self.sizes.next().unwrap();
            let (sequential_iterator, remaining_parallel_iterator) = iterator.iter(next_length);
            self.iterator = Some(remaining_parallel_iterator);
            Some(sequential_iterator)
        }
    }
}

/// Remember how helper threads are helping us.
pub struct Help<I, C> {
    pub(crate) iterator: I,
    pub(crate) help_op: Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
    pub(crate) phantom: PhantomData<C>,
}

impl<C, I> Help<I, C>
where
    C: Send,
    I: ParallelIterator,
{
    pub fn fold<B, F, R>(self, initial_value: B, fold_op: F, retrieve_op: R) -> B
    where
        B: Send,
        F: Fn(B, I::Item) -> B + Sync,
        R: Fn(B, C) -> B + Sync,
    {
        schedule_help(
            self.iterator,
            fold_op,
            self.help_op,
            retrieve_op,
            initial_value,
        )
    }

    pub fn for_each<F, R>(self, op: F, retrieve_op: R)
    where
        F: Fn(I::Item) + Sync,
        R: Fn(C) + Sync,
    {
        self.fold((), |_, e| op(e), |_, c| retrieve_op(c))
    }
}

/// We are going to do one big fold operation in order to compute
/// the final result.
/// Sometimes we fold on some input but sometimes we also fold
/// on intermediate outputs.
/// Having an enumerated type enables to conveniently iterate on both types.
pub(crate) enum RemainingElement<I, C> {
    Input(I),
    Output(C),
}

/// Let's have a sequential thread and helper threads.
pub(crate) fn schedule_help<I, B, C, F, R>(
    mut iterator: I,
    fold_op: F,
    help_op: Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
    retrieve_op: R,
    initial_value: B,
) -> B
where
    B: Send,
    C: Send,
    I: ParallelIterator,
    F: Fn(B, I::Item) -> B + Sync,
    R: Fn(B, C) -> B + Sync,
{
    let stolen_stuffs: &AtomicList<(Option<C>, Option<I>)> = &AtomicList::new();
    rayon::scope(|s| {
        let sizes = iterator.blocks_sizes();
        iterator
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
                RemainingElement::Output(c) => retrieve_op(b, c),
            })
    })
}

fn spawn_stealing_task<'scope, I, C>(
    scope: &Scope<'scope>,
    help_op: &'scope Box<dyn Fn(iter::Flatten<Retriever<I, C>>) -> C + Sync>,
) -> SmallSender<AtomicLink<(Option<C>, Option<I>)>>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
{
    let (sender, receiver) = small_channel();
    scope.spawn(move |s| {
        let stolen_input: Option<AtomicLink<(Option<C>, Option<I>)>>;
        #[cfg(feature = "logs")]
        {
            stolen_input = rayon_logs::subgraph("slave wait", 1, || receiver.recv());
        }
        #[cfg(not(feature = "logs"))]
        {
            stolen_input = receiver.recv();
        }
        if let Some(node) = stolen_input {
            let c = help_op(Retriever::new(s, help_op, &node).flatten());
            let remaining_iterator = node.take().and_then(|c| c.1);
            // store helper's result back in the linked list
            // together with possibly retrieved iterator
            node.replace((Some(c), remaining_iterator))
        }
    });
    sender
}
