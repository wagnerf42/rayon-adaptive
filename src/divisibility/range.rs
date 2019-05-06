//! ranges are divisible too
use crate::prelude::*;
use std::ops::{Range, Sub};

impl<SubOutput, Idx> DivisibleIntoBlocks for Range<Idx>
where
    Idx: Sub<Output = SubOutput> + From<usize> + Copy,
    SubOutput: Into<usize>,
{
    fn base_length(&self) -> Option<usize> {
        Some((self.end - self.start).into())
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        ((self.start..Idx::from(index)), (Idx::from(index)..self.end))
    }
}

impl<Idx, SubOutput> DivisibleAtIndex for Range<Idx>
where
    Idx: Sub<Output = SubOutput> + From<usize> + Copy,
    SubOutput: Into<usize>,
{
}

impl Edible for Range<usize> {
    type Item = usize;
    type SequentialIterator = Range<usize>;
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
        (self.start + size..self.end, self.start..self.start + size)
    }
}

impl Edible for Range<u64> {
    type Item = u64;
    type SequentialIterator = Range<u64>;
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
        (
            self.start + size as u64..self.end,
            self.start..self.start + size as u64,
        )
    }
}
