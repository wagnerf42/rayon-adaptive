//! Parallel iterator on pieces of a `Divisible`. This can be useful when divisions cost nothing.
use crate::prelude::*;
use std::iter::{once, Once};

/// `ParallelIterator` on divided `Divisible`.
pub struct Cut<D> {
    pub(crate) input: D,
}

impl<D: Divisible> Divisible for Cut<D> {
    type Power = D::Power;
    fn base_length(&self) -> Option<usize> {
        self.input.base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.input.divide_at(index);
        (Cut { input: left }, Cut { input: right })
    }
}

impl<D: Divisible + Send> ParallelIterator for Cut<D> {
    type Item = D;
    type SequentialIterator = Once<D>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let left = self.input.divide_on_left_at(size);
        once(left)
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        once(self.input)
    }
}
