use crate::iter::*;
use crate::prelude::*;
use crate::rangefrom::RangeFrom;

type Enumerate<I> = Zip<RangeFrom<usize>, I>;

pub trait IndexedParallelIterator: ParallelIterator<Power = Indexed> {
    fn take(self, n: usize) -> Take<Self> {
        Take { iterator: self, n }
    }
    fn zip<Z>(self, zip_op: Z) -> Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Iter: IndexedParallelIterator,
    {
        Zip {
            a: self,
            b: zip_op.into_par_iter(),
        }
    }
    fn enumerate(self) -> Enumerate<Self> {
        (0usize..).into_par_iter().zip(self)
    }
}

impl<I> IndexedParallelIterator for I where I: ParallelIterator<Power = Indexed> {}
