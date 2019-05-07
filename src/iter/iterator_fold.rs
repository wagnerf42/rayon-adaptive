//! Fold sequential iterators to get a value for each.
//! This simplifies a lot of top-level fold ops (see the code for max as an example).
use crate::prelude::*;
use std::iter::{once, Once};
use std::marker::PhantomData;

/// ParallelIterator where SequentialIterator are turned into a single value.
/// See `iterator_fold` method of `ParallelIterator` trait.
pub struct IteratorFold<
    R: Sized + Send,
    P: Power,
    I: ParallelIterator<P>,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
> {
    pub(crate) iterator: I,
    pub(crate) fold: F,
    pub(crate) phantom: PhantomData<P>,
}

impl<R, P, I, F> Edible for IteratorFold<R, P, I, F>
where
    R: Sized + Send,
    P: Power,
    I: ParallelIterator<P>,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
{
    type Item = R;
    type SequentialIterator = Once<R>;
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
        let (inner_remains, inner_iterator) = self.iterator.iter(size);
        let output = (self.fold)(inner_iterator);
        (
            IteratorFold {
                iterator: inner_remains,
                fold: self.fold,
                phantom: PhantomData,
            },
            once(output),
        )
    }
}
