//! This file contains the most complex scheduling algorithm
//! where one thread works sequentially with no overhead and
//! other threads help him (with overhead).
//! The idea is that if no steal occurs, we end up with the sequential algorithm.
use crate::atomiclist::{AtomicLink, AtomicList};
use crate::prelude::*;
use std::iter;
use std::iter::{once, repeat};
use std::marker::PhantomData;

/// Iterate on sequential iterators until interrupted
/// We are also able to retrieve the remaining part after interruption.
pub struct Taker<'a, I, P> {
    iterator: &'a mut Option<I>, // option is just here to avoid unsafe calls
    interruption_checker: Box<Fn() -> bool>,
    sizes: Box<Iterator<Item = usize>>,
    phantom: PhantomData<P>,
}

impl<'a, I, P> Iterator for Taker<'a, I, P>
where
    P: Power,
    I: ParallelIterator<P> + 'a,
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

fn reduce_until_interrupted<P, I, B, R, C>(
    mut iterator: I,
    reduce: R,
    interruption_checker: C,
) -> (B, I)
where
    P: Power,
    I: ParallelIterator<P>,
    B: Send,
    R: Fn(iter::Flatten<Taker<I, P>>) -> B,
    C: Fn() -> bool + 'static, // for now
{
    let sizes = Box::new(repeat(1)); // for now
    let mut optionned_iterator = Some(iterator);
    let taker = Taker {
        iterator: &mut optionned_iterator,
        interruption_checker: Box::new(interruption_checker),
        sizes,
        phantom: PhantomData,
    };
    let reduced_value = reduce(taker.flatten());
    (reduced_value, optionned_iterator.unwrap())
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
pub(crate) fn schedule_help<P, I, SR, HR, B, R, BH>(
    iterator: I,
    sequential_reducer: SR,
    helper_threads_reducer: HR,
    retrieve_op: R,
) -> B
where
    B: Send,
    P: Power,
    I: ParallelIterator<P>,
    SR: Fn(std::iter::Flatten<Taker<I, P>>) -> B,
    HR: Fn(std::iter::Flatten<Taker<I, P>>) -> BH,
    R: Fn(B, BH) -> B,
{
    let stolen_stuffs: &AtomicList<(Option<BH>, Option<I>)> = &AtomicList::new();
    let sizes = iterator.blocks_sizes();
    rayon::scope(|s| {
        unimplemented!()
        //        let mut todo = iterator
        //            .blocks(sizes)
        //            .flat_map(|block| {
        //                once(RemainingElement::Input(block)).chain(stolen_stuffs.iter().flat_map(
        //                    |(bh, i)| {
        //                        bh.map(RemainingElement::Output)
        //                            .into_iter()
        //                            .chain(i.map(RemainingElement::Input).into_iter())
        //                    },
        //                ))
        //            });
        //        let initial_value = todo.next.unwrap(); // we are sure to have one from sequential thread
        //        todo.fold(b, |b, element| match element {
        //                RemainingElement::Input(i) => unimplemented!(),
        //                RemainingElement::Output(bh) => retrieve_op(b, bh),
        //            })
    })
}
