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
use std::mem;

//TODO: do we really need all these lifetimes ?
/// This structure is used by the sequential worker
/// to answer steal requests.
/// It acts as an iterator on all sequential iterators produced between steal requests.
struct StealAnswerer<'a, 'c, 'scope, I, H, C> {
    scope: &'a Scope<'scope>,
    sizes_bounds: (usize, usize),
    sizes: Box<Iterator<Item = usize>>, // TODO: remove box
    iterator: Option<I>,                // TODO: keep or remove option ? (and be unsafe)
    help_op: &'scope H,
    stolen_stuffs: &'c AtomicList<(Option<C>, Option<I>)>,
    sender: SmallSender<AtomicLink<(Option<C>, Option<I>)>>,
}

impl<'a, 'c, 'scope, I, H, C> StealAnswerer<'a, 'c, 'scope, I, H, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
    H: Fn(std::iter::Flatten<Taker<I, fn() -> bool>>) -> C + Sync,
{
    fn new(
        scope: &'a Scope<'scope>,
        iterator: I,
        help_op: &'scope H,
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

impl<'a, 'c, 'scope, I, H, C> Iterator for StealAnswerer<'a, 'c, 'scope, I, H, C>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
    H: Fn(std::iter::Flatten<Taker<I, fn() -> bool>>) -> C + Sync,
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
                let stolen_node = self.stolen_stuffs.push_front((None, Some(his_half)));
                let mut new_sender = spawn_stealing_task(self.scope, self.help_op);
                mem::swap(&mut new_sender, &mut self.sender);
                new_sender.send(stolen_node);
                iterator = my_half;
            }
            let next_length = self.sizes.next().unwrap();
            let (sequential_iterator, remaining_parallel_iterator) = iterator.iter(next_length);
            self.iterator = Some(remaining_parallel_iterator);
            Some(sequential_iterator)
        }
    }
}

/// Iterate on sequential iterators until interrupted
/// We are also able to retrieve the remaining part after interruption.
pub struct Taker<'a, 'b, I, C> {
    iterator: &'a mut Option<I>, // option is just here to avoid unsafe calls
    interruption_checker: &'b C,
    sizes: Box<Iterator<Item = usize>>,
}

impl<'a, 'b, I, C> Iterator for Taker<'a, 'b, I, C>
where
    I: ParallelIterator + 'a,
    C: Fn() -> bool + 'b,
{
    type Item = I::SequentialIterator;
    fn next(&mut self) -> Option<Self::Item> {
        if self.iterator.base_length().unwrap_or(1) == 0 || (self.interruption_checker)() {
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

fn reduce_until_interrupted<I, B, R, C>(iterator: I, reduce: R, interruption_checker: &C) -> (B, I)
where
    I: ParallelIterator,
    B: Send,
    R: FnOnce(iter::Flatten<Taker<I, C>>) -> B,
    C: Fn() -> bool,
{
    let policy = iterator.policy();
    let (min_size, max_size) = match policy {
        Policy::Adaptive(min_size, max_size) => (min_size, max_size),
        _ => panic!("non adaptive policies are not supported for helper algorithms"),
    };
    let sizes = Box::new(power_sizes(min_size, max_size));
    let mut optionned_iterator = Some(iterator);
    let taker = Taker {
        iterator: &mut optionned_iterator,
        interruption_checker,
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
    H: Fn(iter::Flatten<Taker<I, fn() -> bool>>) -> C + Clone + Send + Sync,
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
    H: Fn(std::iter::Flatten<Taker<I, fn() -> bool>>) -> C + Sync,
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

fn spawn_stealing_task<'scope, H, I, C>(
    scope: &Scope<'scope>,
    help_op: &'scope H,
) -> SmallSender<AtomicLink<(Option<C>, Option<I>)>>
where
    C: Send + 'scope,
    I: ParallelIterator + 'scope,
    H: Fn(std::iter::Flatten<Taker<I, fn() -> bool>>) -> C + Sync,
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

pub(crate) fn helper_reduction<'scope, I, C, H>(
    scope: &Scope<'scope>,
    link: AtomicLink<(Option<C>, Option<I>)>,
    help_op: &'scope H,
) where
    C: Send,
    I: ParallelIterator,
    H: Fn(std::iter::Flatten<Taker<I, fn() -> bool>>) -> C + Sync,
{
    unimplemented!()
}
