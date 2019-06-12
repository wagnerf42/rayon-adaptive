//! Zip structure.

use crate::prelude::*;
use crate::IndexedPower;
use derive_divisible::Divisible;
use std::iter;

/// `Zip` is returned by the `zip` method on parallel iterators.
#[derive(Divisible)]
#[power(IndexedPower)]
#[trait_bounds(A: ParallelIterator<Power=IndexedPower>, B: ParallelIterator<Power=IndexedPower>)]
pub struct Zip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> ParallelIterator for Zip<A, B>
where
    A: ParallelIterator<Power = IndexedPower>,
    B: ParallelIterator<Power = IndexedPower>,
{
    type Item = (A::Item, B::Item);
    type SequentialIterator = iter::Zip<A::SequentialIterator, B::SequentialIterator>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        self.a.extract_iter(size).zip(self.b.extract_iter(size))
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.a.to_sequential().zip(self.b.to_sequential())
    }
}
