//! This file contains the most complex scheduling algorithm
//! where one thread works sequentially with no overhead and
//! other threads help him (with overhead).
//! The idea is that if no steal occurs, we end up with the sequential algorithm.
use crate::atomiclist::{AtomicLink, AtomicList};
use crate::prelude::*;
use crate::small_channel::{small_channel, SmallSender};
use rayon::Scope;
use std::iter;
use std::iter::once;
use std::iter::repeat;

/// Iterate on sequential iterators until interrupted
/// We are also able to retrieve the remaining part after interruption.
pub struct Taker<'a, I> {
    iterator: &'a mut Option<I>, // option is just here to avoid unsafe calls
    interruption_checker: Box<Fn() -> bool>,
    sizes: Box<Iterator<Item = usize>>,
}

impl<'a, I> Iterator for Taker<'a, I>
where
    I: ParallelIterator + 'a,
{
    type Item = I::SequentialIterator;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iterator.base_length().unwrap_or(1) == 0 {
            None
        } else {
            let next_size = self.sizes.next();
            if let Some(size) = next_size {
                let (sequential_iterator, remaining_parallel_iterator) =
                    self.iterator.take().unwrap().iter(size);
                *self.iterator = Some(remaining_parallel_iterator);
                Some(sequential_iterator)
            } else {
                None
            }
        }
    }
}

fn reduce_until_interrupted<I, B, R, C>(
    mut iterator: I,
    reduce: R,
    interruption_checker: C,
) -> (B, I)
where
    I: ParallelIterator,
    B: Send,
    R: FnOnce(iter::Flatten<Taker<I>>) -> B,
    C: Fn() -> bool + 'static, // for now
{
    let sizes = Box::new(repeat(1)); // for now
    let mut optionned_iterator = Some(iterator);
    let taker = Taker {
        iterator: &mut optionned_iterator,
        interruption_checker: Box::new(interruption_checker),
        sizes,
    };
    let reduced_value = reduce(taker.flatten());
    (reduced_value, optionned_iterator.unwrap())
}

/// Remember how helper threads are helping us.
pub struct Help<I, H> {
    pub(crate) iterator: I,
    pub(crate) help_op: H,
}

impl<C, I, H> Help<I, H>
where
    C: Send,
    I: ParallelIterator,
    H: Fn(iter::Flatten<Taker<I>>) -> C + Clone + Send + Sync,
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
enum RemainingElement<I, BH> {
    Input(I),
    Output(BH),
}

/// Let's have a sequential thread and helper threads.
pub(crate) fn schedule_help<I, B, C, F, R, H>(
    mut iterator: I,
    fold_op: F,
    help_op: H,
    retrieve_op: R,
    initial_value: B,
) -> B
where
    B: Send,
    C: Send,
    I: ParallelIterator,
    F: Fn(B, I::Item) -> B + Sync,
    H: Fn(std::iter::Flatten<Taker<I>>) -> C + Sync,
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
                RemainingElement::Input(i) => sequential_fold(s, i, b, &fold_op, &help_op),
                RemainingElement::Output(c) => retrieve_op(b, c),
            })
    })
}

fn spawn_stealing_task<'scope, H, I, C>(
    scope: &Scope<'scope>,
    help_op: &'scope H,
) -> SmallSender<AtomicLink<(Option<C>, Option<I>)>>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
    H: Fn(std::iter::Flatten<Taker<I>>) -> C + Sync,
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
        if stolen_input.is_none() {
            return;
        }
        helper_reduction(s, stolen_input.unwrap(), help_op)
    });
    sender
}

pub(crate) fn sequential_fold<'scope, I, B, C, F, H>(
    scope: &Scope<'scope>,
    iterator: I,
    initial_value: B,
    fold_op: &'scope F,
    help_op: &'scope H,
) -> B
where
    B: Send,
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
    F: Fn(B, I::Item) -> B,
    H: Fn(std::iter::Flatten<Taker<I>>) -> C + Sync,
{
    let mut remaining_iterator = iterator;
    let mut sender = spawn_stealing_task(scope, help_op);
    let mut current_value = initial_value;
    while remaining_iterator.base_length().unwrap_or(1) > 0 {
        let (new_folded_value, iterator) = reduce_until_interrupted(
            remaining_iterator,
            move |i| i.fold(current_value, fold_op),
            || false,
        );
        current_value = new_folded_value;
        remaining_iterator = iterator;
    }
    current_value
}

pub(crate) fn helper_reduction<'scope, I, C, H>(
    scope: &Scope<'scope>,
    link: AtomicLink<(Option<C>, Option<I>)>,
    help_op: &'scope H,
) where
    C: Send,
    I: ParallelIterator,
    H: Fn(std::iter::Flatten<Taker<I>>) -> C,
{
    unimplemented!()
}
