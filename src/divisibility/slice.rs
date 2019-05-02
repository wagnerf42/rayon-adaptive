//! Implement divisibility for slices.

use super::Divisible;

impl<'a, T: 'a> Divisible for &'a [T] {
    fn base_length(&self) -> usize {
        self.len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: 'a> Divisible for &'a mut [T] {
    fn base_length(&self) -> usize {
        self.len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at_mut(mid)
    }
}
