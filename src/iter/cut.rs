//! Parallel iterator on pieces of a `Divisible`. This can be useful when divisions cost nothing.
use crate::prelude::*;
use std::iter::{once, Once};

/// `ParallelIterator` on divided `Divisible`.
pub struct Cut<D> {
    pub(crate) input: Option<D>, // Option is just to avoid unsafe
}

impl<D: Divisible> Divisible for Cut<D> {
    type Power = D::Power;
    fn base_length(&self) -> Option<usize> {
        self.input.as_ref().unwrap().base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.input.unwrap().divide_at(index);
        (Cut { input: Some(left) }, Cut { input: Some(right) })
    }
}

impl<D: Divisible + Send> ParallelIterator for Cut<D> {
    type Item = D;
    type SequentialIterator = Once<D>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let (left, right) = self.input.take().unwrap().divide_at(size);
        self.input = Some(right);
        once(left)
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        once(self.input.unwrap())
    }
}
