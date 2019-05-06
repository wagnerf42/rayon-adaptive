//! Implement divisibility traits for options.
use super::IndexedPower;
use crate::prelude::*;

impl<T> Divisible<IndexedPower> for Option<T> {
    fn base_length(&self) -> Option<usize> {
        if self.is_some() {
            Some(1)
        } else {
            Some(0)
        }
    }
    fn divide(self) -> (Self, Self) {
        (self, None)
    }
}
