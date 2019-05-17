//! Fold sequential iterators to get a value for each.
//! This simplifies a lot of top-level fold ops (see the code for max as an example).
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::iter::{once, Once};
use std::marker::PhantomData;

/// ParallelIterator where SequentialIterator are turned into a single value.
/// See `iterator_fold` method of `ParallelIterator` trait.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P)]
#[item(R)]
#[sequential_iterator(Once<R>)]
#[iterator_extraction(once((self.fold)(i)))]
pub struct IteratorFold<
    R: Sized + Send,
    P: Power,
    I: ParallelIterator<P>,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
> {
    pub(crate) iterator: I,
    #[divide_by(clone)]
    pub(crate) fold: F,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}
