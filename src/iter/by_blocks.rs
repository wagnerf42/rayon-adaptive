//! `ByBlocks` structure for `ParallelIterator::by_blocks`.
use crate::prelude::*;
use crate::Policy;
use std::marker::PhantomData;

/// Iterator which configured to run on macro blocks. See `ParallelIterator::by_blocks`.
pub struct ByBlocks<P, I> {
    pub(crate) sizes_iterator: Option<Box<Iterator<Item = usize> + Send>>,
    pub(crate) iterator: I,
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: ParallelIterator<P>> Divisible<P> for ByBlocks<P, I> {
    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }
    // note: discard sizes iterator if splitting
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iterator.divide_at(index);
        (
            ByBlocks {
                sizes_iterator: None,
                iterator: left,
                phantom: PhantomData,
            },
            ByBlocks {
                sizes_iterator: None,
                iterator: right,
                phantom: PhantomData,
            },
        )
    }
}

impl<P: Power, I: ParallelIterator<P>> Edible for ByBlocks<P, I> {
    type SequentialIterator = I::SequentialIterator;
    type Item = I::Item;
    fn policy(&self) -> Policy {
        self.iterator.policy()
    }
    // we just don't use the sizes iterator
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let (iterator, remaining) = self.iterator.iter(size);
        self.iterator = remaining;
        (iterator, self)
    }
}

impl<P: Power, I: ParallelIterator<P>> ParallelIterator<P> for ByBlocks<P, I> {
    // because we are in a trait we cannot parametrize ByBlocks with the sizes iterator type.
    // It would require an additional associated type in the trait and there are already enough of
    // them.
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.sizes_iterator
            .take()
            .expect("by blocks with no sizes iterator")
    }
}
