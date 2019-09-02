//! Struct for simple fold op on sequential iterators.
use crate::prelude::*;
use std::iter::Empty;

/// Parallel Iterator where all sequential iterators are folded by given function (see
/// `iterator_fold` method).
pub struct IteratorFold<I, F> {
    pub(crate) iterator: I,
    pub(crate) fold_op: F,
}

impl<I: Divisible, F: Clone> Divisible for IteratorFold<I, F> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            IteratorFold {
                iterator: left,
                fold_op: self.fold_op.clone(),
            },
            IteratorFold {
                iterator: right,
                fold_op: self.fold_op,
            },
        )
    }
}

impl<R, I, F> ItemProducer for IteratorFold<I, F>
where
    I: ParallelIterator,
    R: Sized + Send,
    F: for<'e> Fn(<Self as FinitePart<'e>>::SeqIter) -> R + Sync,
{
    type Item = R;
}