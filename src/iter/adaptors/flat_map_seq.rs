//! Implementation of flatmap.
use crate::prelude::*;
use derive_divisible::{Divisible, ParallelIterator};
use std::iter;

#[derive(Divisible, ParallelIterator)]
#[power(<<I as Divisible>::Power as Power>::NotIndexed)]
#[item(PI::Item)]
#[sequential_iterator(iter::FlatMap<I::SequentialIterator, PI, F>)]
#[iterator_extraction(i.flat_map(self.map_op.clone()))]
#[trait_bounds(
    I: ParallelIterator,
    PiItem: Send,
    PI: IntoIterator<Item = PiItem>,
    F: Fn(I::Item) -> PI + Sync + Send + Clone,
)]
/// `FlatMapSeq` is returned by the `flat_map_seq` method on parallel iterators.
pub struct FlatMapSeq<I, F> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) map_op: F,
}
