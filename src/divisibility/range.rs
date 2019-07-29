//! ranges are divisible too
use super::IndexedPower;
use crate::prelude::*;
use std::ops::Range;

impl Divisible for Range<u64> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some((self.end - self.start) as usize)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + index as u64;
        debug_assert!(mid <= self.end);
        ((self.start..mid), (mid..self.end))
    }
}

impl Divisible for Range<usize> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(self.end - self.start)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + index;
        debug_assert!(mid <= self.end);
        ((self.start..mid), (mid..self.end))
    }
}

impl Divisible for Range<u32> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some((self.end - self.start) as usize)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let mid = self.start + index as u32;
        debug_assert!(mid <= self.end);
        ((self.start..mid), (mid..self.end))
    }
}
