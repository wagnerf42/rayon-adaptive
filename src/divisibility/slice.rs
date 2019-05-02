//! Implement divisibility for slices.

use super::{DivisibleAtIndex, DivisibleIntoBlocks};
use std::cmp::min;

// read only slice
impl<'a, T: 'a> DivisibleIntoBlocks for &'a [T] {
    fn base_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at(min(index, self.len()))
    }
}

impl<'a, T: 'a> DivisibleAtIndex for &'a [T] {}

// mutable slice
impl<'a, T: 'a> DivisibleIntoBlocks for &'a mut [T] {
    fn base_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(min(index, self.len()))
    }
}

impl<'a, T: 'a> DivisibleAtIndex for &'a mut [T] {}
