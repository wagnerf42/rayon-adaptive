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
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let (start, end) = self.slice.split_at(size);
        self.slice = end;
        start.iter()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.slice.iter()
    }
}

pub struct IterMut<'a, T: 'a + Sync + Send> {
    slice: Option<&'a mut [T]>,
}

impl<'a, T: 'a + Sync + Send> Divisible for IterMut<'a, T> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(self.slice.as_ref().unwrap().len())
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left, right): (&'a mut [T], &'a mut [T]) = self.slice.take().unwrap().divide_at(index);
        (
            IterMut { slice: Some(left) },
            IterMut { slice: Some(right) },
        )
    }
}

impl<'a, T: 'a + Sync + Send> ParallelIterator for IterMut<'a, T> {
    type Item = &'a mut T;
    type SequentialIterator = slice::IterMut<'a, T>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let (start, end) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(end);
        start.iter_mut()
    }
    fn to_sequential(mut self) -> Self::SequentialIterator {
        self.slice.take().unwrap().iter_mut()
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
        IterMut { slice: Some(self) }
    }
}
