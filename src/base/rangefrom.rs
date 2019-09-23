use crate::base::range::Range;
use crate::prelude::*;

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
            type Iter = Range<$x>;
        }
        impl ParallelIterator for RangeFrom<$x> {
            fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
                let end = self.start + size as $x;
                let borrowed_range = Range {
                    range: self.start..end,
                };
                self.start = end;
                borrowed_range
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
