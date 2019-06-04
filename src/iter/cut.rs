//! Parallel iterator on pieces of a `Divisible`. This can be useful when divisions cost nothing.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator};
use std::iter::{once, Once};

/// `ParallelIterator` on divided `Divisible`.
#[derive(Divisible, IntoIterator)]
#[power(D::Power)]
#[item(D)]
#[trait_bounds(D:Divisible + Send)]
pub struct Cut<D> {
    pub(crate) input: D,
}

impl<D: Divisible + Send> ParallelIterator for Cut<D> {
    type Item = D;
    type SequentialIterator = Once<D>;
    fn extract_iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (left, right) = self.input.divide_at(size);
        (once(left), Cut { input: right })
    }
}
