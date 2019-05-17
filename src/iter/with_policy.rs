//! `WithPolicy` structure for `ParallelIterator::with_policy`.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::{Divisible, IntoIterator};
use std::marker::PhantomData;

/// Iterator remembering which scheduling policy has been set by the user.
#[derive(Divisible, IntoIterator)]
#[power(P)]
#[item(I::Item)]
pub struct WithPolicy<P: Power, I: ParallelIterator<P>> {
    #[divide_by(clone)]
    pub(crate) policy: Policy,
    pub(crate) iterator: I,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: ParallelIterator<P>> ParallelIterator<P> for WithPolicy<P, I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
    fn policy(&self) -> Policy {
        self.policy
    }
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (seq_iterator, remaining) = self.iterator.iter(size);
        (
            seq_iterator,
            WithPolicy {
                policy: self.policy,
                iterator: remaining,
                phantom: PhantomData,
            },
        )
    }
}
