//! Fold sequential iterators to get a value for each.
//! This simplifies a lot of top-level fold ops (see the code for max as an example).
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter::{once, Once};

/// ParallelIterator where SequentialIterator are turned into a single value.
/// See `iterator_fold` method of `ParallelIterator` trait.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(I::Power)]
#[item(R)]
#[sequential_iterator(Once<R>)]
#[iterator_extraction(once((self.fold)(i)))]
#[trait_bounds(
    R: Sized + Send,
    I: ParallelIterator,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
)]
pub struct IteratorFold<I, F> {
    pub(crate) iterator: I,
    #[divide_by(clone)]
    pub(crate) fold: F,
}
