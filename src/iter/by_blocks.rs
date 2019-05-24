//! `ByBlocks` structure for `ParallelIterator::by_blocks`.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::{Divisible, IntoIterator};
use std::iter::empty;

/// Iterator which configured to run on macro blocks. See `ParallelIterator::by_blocks`.
#[derive(Divisible, IntoIterator)]
#[power(I::Power)]
#[item(I::Item)]
pub struct ByBlocks<I: ParallelIterator> {
    #[divide_by(default)]
    pub(crate) sizes_iterator: Option<Box<Iterator<Item = usize> + Send>>,
    pub(crate) iterator: I,
}

impl<I: ParallelIterator> ParallelIterator for ByBlocks<I> {
    type SequentialIterator = I::SequentialIterator;
    type Item = I::Item;
    fn policy(&self) -> Policy {
        self.iterator.policy()
    }
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.sizes_iterator
            .take()
            .unwrap_or_else(|| Box::new(empty()))
    }
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let (iterator, remaining) = self.iterator.iter(size);
        self.iterator = remaining;
        (iterator, self)
    }
}
