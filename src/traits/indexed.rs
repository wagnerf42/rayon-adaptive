use crate::iter::*;
use crate::prelude::*;

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
}

impl<I> IndexedParallelIterator for I where I: ParallelIterator<Power = Indexed> {}
