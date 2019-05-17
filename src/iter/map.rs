//! Map iterator.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter;
use std::marker::PhantomData;

/// Map iterator adapter, returning by `map` function on `ParallelIterator`.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P)]
#[item(R)]
#[sequential_iterator(iter::Map<I::SequentialIterator, F>)]
#[iterator_extraction(i.map(self.f.clone()))]
pub struct Map<R: Send, P: Power, I: ParallelIterator<P>, F: Fn(I::Item) -> R + Clone + Send> {
    pub(crate) iter: I,
    #[divide_by(clone)]
    pub(crate) f: F,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}
