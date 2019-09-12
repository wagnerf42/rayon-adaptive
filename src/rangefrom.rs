use crate::prelude::*;

pub struct RangeFrom<Idx> {
    start: Idx,
}

// sadly we cannot use ParRange since the owner would not be us.
pub struct BorrowedRangeFrom<Idx> {
    range: std::ops::Range<Idx>,
}

macro_rules! implement_traits {
    ($x: ty) => {
        impl ItemProducer for RangeFrom<$x> {
            type Item = $x;
            type Owner = Self;
            type Power = Indexed;
        }

        impl ItemProducer for BorrowedRangeFrom<$x> {
            type Item = $x;
            type Owner = RangeFrom<$x>;
            type Power = Indexed;
        }

        impl<'e> Borrowed<'e> for RangeFrom<$x> {
            type ParIter = BorrowedRangeFrom<$x>;
            type SeqIter = std::ops::Range<$x>;
        }

        impl Divisible for BorrowedRangeFrom<$x> {
            fn is_divisible(&self) -> bool {
                self.range.len() > 1
            }
            fn divide(self) -> (Self, Self) {
                let mid = self.range.len() / 2;
                let end = self.range.start + mid as $x;
                (
                    BorrowedRangeFrom {
                        range: self.range.start..end,
                    },
                    BorrowedRangeFrom {
                        range: end..self.range.end,
                    },
                )
            }
        }

        impl ParallelIterator for RangeFrom<$x> {
            fn borrow_on_left_for<'e>(
                &'e mut self,
                size: usize,
            ) -> <Self::Owner as Borrowed<'e>>::ParIter {
                let old_start = self.start;
                self.start += size as $x;
                BorrowedRangeFrom {
                    range: old_start..self.start,
                }
            }
            fn sequential_borrow_on_left_for<'e>(
                &'e mut self,
                size: usize,
            ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
                let old_start = self.start;
                self.start += size as $x;
                old_start..self.start
            }
        }

        impl ParallelIterator for BorrowedRangeFrom<$x> {
            fn borrow_on_left_for<'e>(
                &'e mut self,
                size: usize,
            ) -> <Self::Owner as Borrowed<'e>>::ParIter {
                let old_start = self.range.start;
                self.range.start += size as $x;
                BorrowedRangeFrom {
                    range: old_start..self.range.start,
                }
            }
            fn sequential_borrow_on_left_for<'e>(
                &'e mut self,
                size: usize,
            ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
                let old_start = self.range.start;
                self.range.start += size as $x;
                old_start..self.range.start
            }
        }

        impl FiniteParallelIterator for BorrowedRangeFrom<$x> {
            fn len(&self) -> usize {
                self.range.len()
            }
        }

        impl IntoParallelIterator for std::ops::RangeFrom<$x> {
            type Iter = RangeFrom<$x>;
            type Item = $x;
            fn into_par_iter(self) -> Self::Iter {
                RangeFrom { start: self.start }
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
