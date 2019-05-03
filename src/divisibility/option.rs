//! Implement divisibility traits for options.
use super::{DivisibleAtIndex, DivisibleIntoBlocks};

impl<T> DivisibleIntoBlocks for Option<T> {
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

impl<T> DivisibleAtIndex for Option<T> {}
