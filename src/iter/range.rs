//! Range are parallel iterators.
use crate::divisibility::IndexedPower;
use crate::prelude::*;
use std::ops::Range;

/// `ParallelIterator` on `Range`.
pub struct RangeParIter<E>(Range<E>);

impl Divisible for RangeParIter<u64> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        self.0.base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.0.divide_at(index);
        (RangeParIter(left), RangeParIter(right))
    }
}

impl Divisible for RangeParIter<usize> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        self.0.base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.0.divide_at(index);
        (RangeParIter(left), RangeParIter(right))
    }
}

impl ParallelIterator for RangeParIter<usize> {
    type Item = usize;
    type SequentialIterator = Range<usize>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (iterator, remaining) = self.0.divide_at(size);
        (iterator, RangeParIter(remaining))
    }
}

impl ParallelIterator for RangeParIter<u64> {
    type Item = u64;
    type SequentialIterator = Range<u64>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (iterator, remaining) = self.0.divide_at(size);
        (iterator, RangeParIter(remaining))
    }
}

impl IntoParallelIterator for Range<usize> {
    type Iter = RangeParIter<usize>;
    type Item = usize;
    fn into_par_iter(self) -> Self::Iter {
        RangeParIter(self)
    }
}

impl IntoParallelIterator for Range<u64> {
    type Iter = RangeParIter<u64>;
    type Item = u64;
    fn into_par_iter(self) -> Self::Iter {
        RangeParIter(self)
    }
}
