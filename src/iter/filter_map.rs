//! `FilterMap`
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;
use std::marker::PhantomData;

/// `FilterMap` struct is obtained from `filter_map` method on `ParallelIterator`.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P::NotIndexed)]
#[item(R)]
#[iterator_extraction(i.filter_map(self.filter_op.clone()))]
#[sequential_iterator(iter::FilterMap<I::SequentialIterator, Predicate>)]
#[trait_bounds(
    P: Power,
    R: Send,
    I: ParallelIterator<P>,
    Predicate: Fn(I::Item) -> Option<R> + Clone + Send,
)]
pub struct FilterMap<P, I, Predicate> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) filter_op: Predicate,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}
