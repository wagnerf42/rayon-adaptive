//! Implementation of flatmap.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;
use std::marker::PhantomData;

#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P::NotIndexed)]
#[item(PI::Item)]
#[sequential_iterator(iter::FlatMap<I::SequentialIterator, PI, F>)]
#[iterator_extraction(i.flat_map(self.map_op.clone()))]
#[trait_bounds(
    P: Power,
    I: ParallelIterator<P>,
    PiItem: Send,
    PI: IntoIterator<Item = PiItem>,
    F: Fn(I::Item) -> PI + Sync + Send + Clone,
)]
/// `FlatMap` is returned by the `flat_map` method on parallel iterators.
pub struct FlatMap<P, I, F> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) map_op: F,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}
