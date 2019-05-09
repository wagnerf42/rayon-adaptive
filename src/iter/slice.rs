//! Slices are parallel iterators.

use crate::prelude::*;
use std::slice::Iter;

impl<'a, T: 'a + Sync> Edible for &'a [T] {
    type Item = &'a T;
    type SequentialIterator = Iter<'a, T>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (beginning, remaining) = self.divide_at(size);
        (beginning.iter(), remaining)
    }
}
