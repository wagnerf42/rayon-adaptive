//! Struct for simple fold op on sequential iterators.
use crate::dislocated::Dislocated;
use crate::prelude::*;
use std::iter::{once, Once};

// we start with the parallel type

/// Parallel Iterator where all sequential iterators are folded by given function (see
/// `iterator_fold` method).
pub struct IteratorFold<I, F> {
    pub(crate) iterator: I,
    pub(crate) fold_op: F,
}

impl<R, I, F> ItemProducer for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    type Owner = IteratorFold<I::Owner, F>;
    type Item = R;
}

impl<'e, R, I, F> Borrowed<'e> for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    type ParIter = BorrowedIteratorFold<'e, <I::Owner as Borrowed<'e>>::ParIter, F>;
    type SeqIter = Once<R>;
}

impl<R, I, F> ParallelIterator for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowedIteratorFold {
            iterator: self.iterator.borrow_on_left_for(size),
            fold_op: Dislocated::new(&self.fold_op),
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        let i = self.iterator.sequential_borrow_on_left_for(size);
        let r = (self.fold_op)(i);
        once(r)
    }
}

// we continue with the borrowed parallel type

pub struct BorrowedIteratorFold<'e, I, F: Sync> {
    iterator: I,
    fold_op: Dislocated<'e, F>,
}

impl<'a, R, I, F> ItemProducer for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    type Owner = IteratorFold<I::Owner, F>;
    type Item = R;
}

impl<'a, R, I, F> Divisible for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            BorrowedIteratorFold {
                iterator: left,
                fold_op: self.fold_op,
            },
            BorrowedIteratorFold {
                iterator: right,
                fold_op: self.fold_op,
            },
        )
    }
}

impl<'a, R, I, F> ParallelIterator for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowedIteratorFold {
            iterator: self.iterator.borrow_on_left_for(size),
            fold_op: self.fold_op,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        let i = self.iterator.sequential_borrow_on_left_for(size);
        let r = (self.fold_op)(i);
        once(r)
    }
}

impl<'a, R, I, F> FiniteParallelIterator for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

impl<R, I, F> FiniteParallelIterator for IteratorFold<I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I::Owner as Borrowed>::SeqIter) -> R + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
