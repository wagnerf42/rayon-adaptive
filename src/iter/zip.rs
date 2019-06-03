//! Zip structure.

use crate::prelude::*;
use crate::IndexedPower;
use derive_divisible::{Divisible, IntoIterator};
use std::iter;

/// `Zip` is returned by the `zip` method on parallel iterators.
#[derive(Divisible, IntoIterator)]
#[item((A::Item, B::Item))]
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
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (iter_a, remaining_a) = self.a.iter(size);
        let (iter_b, remaining_b) = self.b.iter(size);
        (
            iter_a.zip(iter_b),
            Zip {
                a: remaining_a,
                b: remaining_b,
            },
        )
    }
}
