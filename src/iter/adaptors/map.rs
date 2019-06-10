//! Map iterator.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;

/// Map iterator adapter, returning by `map` function on `ParallelIterator`.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(I::Power)]
#[item(R)]
#[sequential_iterator(iter::Map<I::SequentialIterator, F>)]
#[iterator_extraction(i.map(self.f.clone()))]
#[trait_bounds(R: Send, I: ParallelIterator, F: Fn(I::Item) -> R + Clone + Send)]
pub struct Map<I, F> {
    pub(crate) iter: I,
    #[divide_by(clone)]
    pub(crate) f: F,
}
