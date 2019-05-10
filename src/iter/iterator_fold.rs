//! Fold sequential iterators to get a value for each.
//! This simplifies a lot of top-level fold ops (see the code for max as an example).
use crate::prelude::*;
use crate::Policy;
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
    fn policy(&self) -> Policy {
        self.iterator.policy()
    }
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (inner_iterator, inner_remains) = self.iterator.iter(size);
        let output = (self.fold)(inner_iterator);
        (
            once(output),
            IteratorFold {
                iterator: inner_remains,
                fold: self.fold,
                phantom: PhantomData,
            },
        )
    }
}

impl<R, P, I, F> Divisible<P> for IteratorFold<R, P, I, F>
where
    R: Sized + Send,
    P: Power,
    I: ParallelIterator<P>,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
{
    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left_iterator, right_iterator) = self.iterator.divide_at(index);
        (
            IteratorFold {
                iterator: left_iterator,
                fold: self.fold.clone(),
                phantom: PhantomData,
            },
            IteratorFold {
                iterator: right_iterator,
                fold: self.fold,
                phantom: PhantomData,
            },
        )
    }
}

impl<R, P, I, F> ParallelIterator<P> for IteratorFold<R, P, I, F>
where
    R: Sized + Send,
    P: Power,
    I: ParallelIterator<P>,
    F: Fn(I::SequentialIterator) -> R + Send + Clone,
{
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iterator.blocks_sizes()
    }
}
