//! Slices are parallel iterators.

use crate::divisibility::IndexedPower;
use crate::prelude::*;
use std::slice::{Iter, IterMut};

impl<'a, T: 'a + Sync> ParallelIterator<IndexedPower> for &'a [T] {
    type Item = &'a T;
    type SequentialIterator = Iter<'a, T>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (beginning, remaining) = self.divide_at(size);
        (beginning.iter(), remaining)
    }
}

impl<'a, T: 'a + Sync + Send> ParallelIterator<IndexedPower> for &'a mut [T] {
    type Item = &'a mut T;
    type SequentialIterator = IterMut<'a, T>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (beginning, remaining) = self.divide_at(size);
        (beginning.iter_mut(), remaining)
    }
}
