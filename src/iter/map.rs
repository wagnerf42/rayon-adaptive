//! Map iterator.
use crate::prelude::*;
use crate::Policy;
use std::iter;
use std::marker::PhantomData;

/// Map iterator adapter, returning by `map` function on `ParallelIterator`.
pub struct Map<P, I, F> {
    pub(crate) iter: I,
    pub(crate) f: F,
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: ParallelIterator<P>, R: Send, F: Fn(I::Item) -> R + Send + Clone> Edible
    for Map<P, I, F>
{
    type Item = R;
    type SequentialIterator = iter::Map<I::SequentialIterator, F>;
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let (sequential_iterator, remaining) = self.iter.iter(size);
        self.iter = remaining;
        (sequential_iterator.map(self.f.clone()), self)
    }
    fn policy(&self) -> Policy {
        self.iter.policy()
    }
}

impl<P: Power, I: ParallelIterator<P>, R: Send, F: Fn(I::Item) -> R + Send + Clone> Divisible<P>
    for Map<P, I, F>
{
    fn base_length(&self) -> Option<usize> {
        self.iter.base_length()
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.divide_at(index);
        self.iter = left;
        let right_part = Map {
            iter: right,
            f: self.f.clone(),
            phantom: PhantomData,
        };
        (self, right_part)
    }
}

impl<P: Power, I: ParallelIterator<P>, R: Send, F: Fn(I::Item) -> R + Send + Clone>
    ParallelIterator<P> for Map<P, I, F>
{
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iter.blocks_sizes()
    }
}
