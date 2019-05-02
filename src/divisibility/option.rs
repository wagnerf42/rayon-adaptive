//! Implement divisibility traits for options.
use super::Divisible;

impl<T> Divisible for Option<T> {
    fn base_length(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }
    fn divide(self) -> (Self, Self) {
        (self, None)
    }
}
