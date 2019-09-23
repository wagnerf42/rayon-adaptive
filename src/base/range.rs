use crate::prelude::*;

#[derive(Debug)]
pub struct Range<Idx> {
    pub(crate) range: std::ops::Range<Idx>,
}

macro_rules! implement_traits {
    ($x: ty) => {
        impl ItemProducer for Range<$x> {
            type Item = $x;
        }
        impl Powered for Range<$x> {
            type Power = Indexed;
        }
        impl<'e> ParBorrowed<'e> for Range<$x> {
            type Iter = Range<$x>;
        }
        impl<'e> SeqBorrowed<'e> for Range<$x> {
            type Iter = std::ops::Range<$x>;
        }
        impl Divisible for Range<$x> {
            fn should_be_divided(&self) -> bool {
                self.range.len() > 1
            }
            fn divide(mut self) -> (Self, Self) {
                let mid = ((self.range.start + self.range.end) / 2) as $x;
                let right = Range {
                    range: mid..self.range.end,
                };
                self.range.end = mid;
                (self, right)
            }
        }
        impl BorrowingParallelIterator for Range<$x> {
            fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
                let mid = self.range.start + size as $x;
                let left = self.range.start..mid;
                self.range.start = mid;
                left
            }
            fn iterations_number(&self) -> usize {
                self.range.len()
            }
        }
        impl ParallelIterator for Range<$x> {
            fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
                let mid = self.range.start + size as $x;
                let left = Range {
                    range: self.range.start..mid,
                };
                self.range.start = mid;
                left
            }
            fn bound_iterations_number(&self, size: usize) -> usize {
                std::cmp::min(self.range.len(), size)
            }
        }

        impl IntoParallelIterator for std::ops::Range<$x> {
            type Iter = Range<$x>;
            type Item = $x;
            fn into_par_iter(self) -> Self::Iter {
                Range { range: self }
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
