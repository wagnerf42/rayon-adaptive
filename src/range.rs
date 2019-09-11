use crate::prelude::*;
use std::ops::Range;

pub struct ParRange {
    pub range: Range<u32>,
}

impl Divisible for ParRange {
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

impl FiniteParallelIterator for ParRange {
    fn len(&self) -> usize {
        self.range.len()
    }
}

impl ParallelIterator for ParRange {
    fn borrow_on_left_for<'e>(&mut self, size: usize) -> ParRange {
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

impl<'e> Borrowed<'e> for ParRange {
    type ParIter = ParRange;
    type SeqIter = Range<u32>;
}

impl ItemProducer for ParRange {
    type Owner = Self;
    type Item = u32;
}
