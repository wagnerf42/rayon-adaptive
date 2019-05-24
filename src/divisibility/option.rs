//! Implement divisibility traits for options.
use super::IndexedPower;
use crate::prelude::*;

impl<T> Divisible for Option<T> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        if self.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
    fn divide_at(self, _index: usize) -> (Self, Self) {
        (self, None)
    }
}
