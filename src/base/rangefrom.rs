use crate::prelude::*;
use crate::traits::Adaptive;

pub struct RangeFrom<Idx> {
    start: Idx,
}

macro_rules! implement_traits {
    ($x: ty) => {
        impl ItemProducer for RangeFrom<$x> {
            type Item = $x;
        }
        impl Powered for RangeFrom<$x> {
            type Power = Indexed;
        }
        impl<'e> ParBorrowed<'e> for RangeFrom<$x> {
            type Iter = DivisibleIter<std::ops::Range<$x>, Adaptive>;
        }
        impl ParallelIterator for RangeFrom<$x> {
            fn bound_iterations_number(&self, size: usize) -> usize {
                size
            }
            fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
                let end = self.start + size as $x;
                let borrowed_range = DivisibleIter {
                    base: self.start..end,
                    schedule_type: Adaptive {},
                };
                self.start = end;
                borrowed_range
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
