use crate::prelude::*;
use std::ops::Range;

pub struct ParRange<Idx> {
    pub range: Range<Idx>,
}

macro_rules! implement_traits {
    ($x: ty) => {
        impl Divisible for ParRange<$x> {
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

        impl FiniteParallelIterator for ParRange<$x> where {
            fn len(&self) -> usize {
                self.range.len()
            }
        }

        impl ParallelIterator for ParRange<$x> {
            fn borrow_on_left_for<'e>(&mut self, size: usize) -> ParRange<$x> {
                let start = self.range.start;
                self.range.start += size as $x;
                ParRange {
                    range: start..self.range.start,
                }
            }
            fn sequential_borrow_on_left_for<'e>(&mut self, size: usize) -> Range<$x> {
                let start = self.range.start;
                self.range.start += size as $x;
                start..self.range.start
            }
        }

        impl<'e> Borrowed<'e> for ParRange<$x> {
            type ParIter = ParRange<$x>;
            type SeqIter = Range<$x>;
        }

        impl ItemProducer for ParRange<$x> {
            type Owner = Self;
            type Item = $x;
            type Power = Indexed;
        }

        impl IntoParallelIterator for Range<$x> {
            type Iter = ParRange<$x>;
            type Item = $x;
            fn into_par_iter(self) -> Self::Iter {
                ParRange { range: self }
            }
        }
    };
}

implement_traits!(i16);
implement_traits!(u16);
implement_traits!(i32);
implement_traits!(isize);
implement_traits!(u8);
implement_traits!(usize);
implement_traits!(i8);
implement_traits!(u32);
