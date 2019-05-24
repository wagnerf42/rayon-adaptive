//! This file contains the most complex scheduling algorithm
//! where one thread works sequentially with no overhead and
//! other threads help him (with overhead).
//! The idea is that if no steal occurs, we end up with the sequential algorithm.
use crate::prelude::*;
use std::iter;
use std::iter::repeat;
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
