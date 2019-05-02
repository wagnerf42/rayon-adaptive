//! ranges are divisible too
use super::{DivisibleAtIndex, DivisibleIntoBlocks};
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
