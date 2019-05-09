//! `WithPolicy` structure for `ParallelIterator::with_policy`.
use crate::prelude::*;
use crate::Policy;
use std::marker::PhantomData;

/// Iterator remembering which scheduling policy has been set by the user.
pub struct WithPolicy<P, I> {
    pub(crate) policy: Policy,
    pub(crate) iterator: I,
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: ParallelIterator<P>> Edible for WithPolicy<P, I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
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

impl<P: Power, I: ParallelIterator<P>> Divisible<P> for WithPolicy<P, I> {
    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left_iterator, right_iterator) = self.iterator.divide_at(index);
        (
            WithPolicy {
                policy: self.policy,
                iterator: left_iterator,
                phantom: PhantomData,
            },
            WithPolicy {
                policy: self.policy,
                iterator: right_iterator,
                phantom: PhantomData,
            },
        )
    }
    fn policy(&self) -> Policy {
        self.policy
    }
}
