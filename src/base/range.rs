use crate::prelude::*;

macro_rules! implement_traits {
    ($x: ty) => {
        impl DivisibleParallelIterator for std::ops::Range<$x> {
            fn base_length(&self) -> usize {
                self.len()
            }
            fn cut_at_index(&mut self, index: usize) -> Self {
                let left = self.start..self.start + (index as $x);
                self.start = left.end;
                left
            }
        }
        impl IntoParallelIterator for std::ops::Range<$x> {
            type Iter = DivisibleIter<std::ops::Range<$x>>;
            type Item = $x;
            fn into_par_iter(self) -> Self::Iter {
                DivisibleIter { base: self }
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
