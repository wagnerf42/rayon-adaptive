use crate::prelude::*;
use std::ops::Range;

pub struct ParRange<Idx> {
    pub range: Range<Idx>,
}

impl Divisible for ParRange<u32> {
    fn is_divisible(&self) -> bool {
        self.range.len() > 1
    }
    fn divide(self) -> (Self, Self) {
        let mid = (self.range.start + self.range.end) / 2;
        (
            ParRange {
                range: self.range.start..mid,
            },
            ParRange {
                range: mid..self.range.end,
            },
        )
    }
}

impl FiniteParallelIterator for ParRange<u32> where {
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl ParallelIterator for ParRange<u32> {
    fn borrow_on_left_for<'e>(&mut self, size: usize) -> ParRange<u32> {
        let start = self.range.start;
        self.range.start += size as u32;
        ParRange {
            range: start..self.range.start,
        }
    }
    fn sequential_borrow_on_left_for<'e>(&mut self, size: usize) -> Range<u32> {
        let start = self.range.start;
        self.range.start += size as u32;
        start..self.range.start
    }
}

impl<'e> Borrowed<'e> for ParRange<u32> {
    type ParIter = ParRange<u32>;
    type SeqIter = Range<u32>;
}

impl ItemProducer for ParRange<u32> {
    type Owner = Self;
    type Item = u32;
    type Power = Indexed;
}

impl IntoParallelIterator for Range<u32> {
    type Iter = ParRange<u32>;
    type Item = u32;
    fn into_par_iter(self) -> Self::Iter {
        ParRange { range: self }
    }
}
