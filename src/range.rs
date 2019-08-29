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
    type SequentialIterator = Range<u32>;
    fn len(&self) -> usize {
        self.range.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.range
    }
}

impl ParallelIterator for ParRange {
    fn borrow_on_left_for<'extraction>(&mut self, size: usize) -> ParRange {
        let start = self.range.start;
        self.range.start += size as u32;
        ParRange {
            range: start..self.range.start,
        }
    }
}

impl<'extraction> FinitePart<'extraction> for ParRange {
    type Iter = ParRange;
}

impl ItemProducer for ParRange {
    type Item = u32;
}
