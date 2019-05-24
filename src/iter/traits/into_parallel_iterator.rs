//! `IntoParallelIterator` trait
use crate::prelude::*;

/// Turn something into a parallel iterator.
pub trait IntoParallelIterator {
    /// This is the type of iterators we get.
    type Iter: ParallelIterator<Item = Self::Item>;
    /// This is the type of items we loop on.
    type Item: Send;
    /// Change into a parallel iterator.
    fn into_par_iter(self) -> Self::Iter;
}

// all parallel iterators can be converted to themselves
impl<I: ParallelIterator> IntoParallelIterator for I {
    type Item = I::Item;
    type Iter = I;
    fn into_par_iter(self) -> Self::Iter {
        self
    }
}
