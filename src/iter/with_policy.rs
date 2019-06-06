//! `WithPolicy` structure for `ParallelIterator::with_policy`.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::{Divisible, IntoIterator};

/// Iterator remembering which scheduling policy has been set by the user.
#[derive(Divisible, IntoIterator)]
#[power(I::Power)]
#[item(I::Item)]
#[trait_bounds(I: ParallelIterator)]
pub struct WithPolicy<I> {
    #[divide_by(clone)]
    pub(crate) policy: Policy,
    pub(crate) iterator: I,
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
