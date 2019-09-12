use crate::prelude::*;
use crate::iter::*;

pub trait IndexedParallelIterator: ParallelIterator<Power = Indexed> {
    fn take(self, n: usize) -> Take<Self> {
        Take { iterator: self, n }
    }
    //TODO: use IntoParallelIterator
    fn zip<B: IndexedParallelIterator>(self, zip_op: B) -> Zip<Self, B> {
        Zip { a: self, b: zip_op }
    }
}

impl<I> IndexedParallelIterator for I where I: ParallelIterator<Power = Indexed> {}
