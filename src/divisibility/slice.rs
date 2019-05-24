//! Implement divisibility for slices.

use super::IndexedPower;
use crate::prelude::*;
use std::cmp::min;

// read only slice
impl<'a, T: 'a> Divisible for &'a [T] {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at(min(index, self.len()))
    }
}

// mutable slice
impl<'a, T: 'a> Divisible for &'a mut [T] {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(self.len())
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(min(index, self.len()))
    }
}
