//! ranges are divisible too
use super::IndexedPower;
use crate::prelude::*;
use std::ops::Range;

impl Divisible<IndexedPower> for Range<u64> {
    fn base_length(&self) -> Option<usize> {
        Some((self.end - self.start) as usize)
    }
    fn divide(self) -> (Self, Self) {
        let index = (self.start + self.end) / 2;
        ((self.start..index), (index..self.end))
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        ((self.start..index as u64), (index as u64..self.end))
    }
}

impl Divisible<IndexedPower> for Range<usize> {
    fn base_length(&self) -> Option<usize> {
        Some(self.end - self.start)
    }
    fn divide(self) -> (Self, Self) {
        let index = (self.start + self.end) / 2;
        ((self.start..index), (index..self.end))
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        ((self.start..index), (index..self.end))
    }
}
