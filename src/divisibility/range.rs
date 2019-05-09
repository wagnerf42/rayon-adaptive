//! ranges are divisible too
use super::IndexedPower;
use crate::prelude::*;
use std::ops::Range;

impl Divisible<IndexedPower> for Range<u64> {
    fn base_length(&self) -> Option<usize> {
        Some((self.end - self.start) as usize)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + index as u64;
        ((self.start..mid), (mid..self.end))
    }
}

impl Divisible<IndexedPower> for Range<usize> {
    fn base_length(&self) -> Option<usize> {
        Some(self.end - self.start)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + index;
        ((self.start..mid), (mid..self.end))
    }
}
