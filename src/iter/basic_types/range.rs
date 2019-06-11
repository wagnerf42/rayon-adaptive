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
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let end = self.0.start + size;
        let iter = self.0.start..end;
        self.0.start = end;
        iter
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.0
    }
}

impl ParallelIterator for RangeParIter<u64> {
    type Item = u64;
    type SequentialIterator = Range<u64>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        debug_assert!(self.base_length().map(|l| l >= size).unwrap_or(true));
        let end = self.0.start + size as u64;
        let iter = self.0.start..end;
        self.0.start = end;
        iter
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.0
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
