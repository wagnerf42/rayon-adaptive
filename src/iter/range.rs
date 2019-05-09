//! Range are parallel iterators.
use crate::prelude::*;
use std::ops::Range;

impl Edible for Range<usize> {
    type Item = usize;
    type SequentialIterator = Range<usize>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        self.divide_at(size)
    }
}

impl Edible for Range<u64> {
    type Item = u64;
    type SequentialIterator = Range<u64>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        self.divide_at(size)
    }
}
