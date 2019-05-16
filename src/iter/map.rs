//! Map iterator.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::Divisible;
use std::iter;
use std::marker::PhantomData;

/// Map iterator adapter, returning by `map` function on `ParallelIterator`.
#[derive(Divisible)]
#[power(P)]
pub struct Map<P: Power, I: Divisible<P>, F: Clone> {
    pub(crate) iter: I,
    #[divide_by(clone)]
    pub(crate) f: F,
    #[divide_by(default)]
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

impl<P: Power, I: ParallelIterator<P>, R: Send, F: Fn(I::Item) -> R + Send + Clone>
    ParallelIterator<P> for Map<P, I, F>
{
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iter.blocks_sizes()
    }
}
