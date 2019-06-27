//! `WithPolicy` structure for `ParallelIterator::with_policy`.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::Divisible;

/// Iterator remembering which scheduling policy has been set by the user.
#[derive(Divisible)]
#[power(I::Power)]
#[trait_bounds(I: ParallelIterator)]
pub struct WithPolicy<I> {
    /// scheduling policy
    #[divide_by(clone)]
    pub policy: Policy,
    /// inner iterator
    pub iterator: I,
}

impl<I: ParallelIterator> ParallelIterator for WithPolicy<I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
    fn policy(&self) -> Policy {
        self.policy
    }
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        self.iterator.extract_iter(size)
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential()
    }
}
