//! Slices are parallel iterators.

use crate::divisibility::IndexedPower;
use crate::prelude::*;
use derive_divisible::Divisible;
use std::slice;

//TODO: deriving divisible does not work with a tuple struct
#[derive(Divisible)]
#[power(IndexedPower)]
pub struct Iter<'a, T: 'a + Sync> {
    slice: &'a [T],
}

impl<'a, T: 'a + Sync> ParallelIterator for Iter<'a, T> {
    type Item = &'a T;
    type SequentialIterator = slice::Iter<'a, T>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (beginning, remaining) = self.divide_at(size);
        (beginning.slice.iter(), remaining)
    }
}

#[derive(Divisible)]
#[power(IndexedPower)]
pub struct IterMut<'a, T: 'a + Sync + Send> {
    slice: &'a mut [T],
}

impl<'a, T: 'a + Sync + Send> ParallelIterator for IterMut<'a, T> {
    type Item = &'a mut T;
    type SequentialIterator = slice::IterMut<'a, T>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (beginning, remaining) = self.divide_at(size);
        (beginning.slice.iter_mut(), remaining)
    }
}

impl<'a, T: 'a + Sync> IntoParallelIterator for &'a [T] {
    type Iter = Iter<'a, T>;
    type Item = &'a T;
    fn into_par_iter(self) -> Self::Iter {
        Iter { slice: self }
    }
}

impl<'a, T: 'a + Sync + Send> IntoParallelIterator for &'a mut [T] {
    type Iter = IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_par_iter(self) -> Self::Iter {
        IterMut { slice: self }
    }
}
