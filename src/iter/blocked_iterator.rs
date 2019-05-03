//! `BlockedIterator` structure. Like `ParallelIterator` but more features
//! due to a more powerful divisibility.

use super::{BaseIterator, Edible};
use crate::divisibility::BlocksIterator;
use crate::prelude::*;
use std::iter::Map;

/// BlockedIterator is a struct here, not a trait.
/// Doing that enables us an easy specialisation code.
pub struct BlockedIterator<Input: DivisibleIntoBlocks + Edible>(Input);

impl<Input: DivisibleIntoBlocks + Edible> Divisible for BlockedIterator<Input> {
    fn base_length(&self) -> usize {
        self.0.base_length().expect("folding an infinite iterator")
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.base_length() / 2;
        let (left, right) = self.0.divide_at(mid);
        (BlockedIterator(left), BlockedIterator(right))
    }
}

impl<Input: DivisibleIntoBlocks + Edible> Edible for BlockedIterator<Input> {
    type SequentialIterator = Input::SequentialIterator;
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
        let (remaining_input, iterator) = self.0.iter(size);
        (BlockedIterator(remaining_input), iterator)
    }
}

impl<Input: DivisibleIntoBlocks + Edible> BaseIterator for BlockedIterator<Input> {
    type BlocksIterator = Map<BlocksIterator<Input>, fn(Input) -> BlockedIterator<Input>>;
    fn blocks(self) -> Self::BlocksIterator {
        self.0.blocks().map(BlockedIterator)
    }
}

impl<InItem, InIterator, Input> BlockedIterator<Input>
where
    InIterator: Iterator<Item = InItem>,
    Input: DivisibleIntoBlocks + Edible<SequentialIterator = InIterator>,
{
    fn find_first(self) -> Option<InItem> {
        unimplemented!()
    }
}
