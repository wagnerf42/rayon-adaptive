//! `FilterMap`
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;

/// `FilterMap` struct is obtained from `filter_map` method on `ParallelIterator`.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(<<I as Divisible>::Power as Power>::NotIndexed)]
#[item(R)]
#[iterator_extraction(i.filter_map(self.filter_op.clone()))]
#[sequential_iterator(iter::FilterMap<I::SequentialIterator, Predicate>)]
#[trait_bounds(
    R: Send,
    I: ParallelIterator,
    Predicate: Fn(I::Item) -> Option<R> + Clone + Send,
)]
pub struct FilterMap<I, Predicate> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) filter_op: Predicate,
}
