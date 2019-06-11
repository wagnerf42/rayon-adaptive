//! Implementation of the `Filter` parallel iterator.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;

/// `Filter` parallel iterator structure obtained from the `filter` method.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(<<I as Divisible>::Power as Power>::NotIndexed)]
#[item(I::Item)]
#[sequential_iterator(iter::Filter<I::SequentialIterator, P>)]
#[iterator_extraction(i.filter(self.filter_op.clone()))]
#[trait_bounds(I: ParallelIterator, P: Fn(&I::Item) -> bool + Clone + Send)]
pub struct Filter<I, P> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) filter_op: P,
}
