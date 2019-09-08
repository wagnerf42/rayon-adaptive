//! Struct for simple fold op on sequential iterators.
use crate::dislocated::Dislocated;
use crate::prelude::*;
use std::iter::{once, Once};

// // we start with the parallel type
//
// /// Parallel Iterator where all sequential iterators are folded by given function (see
// /// `iterator_fold` method).
// pub struct IteratorFold<I, F> {
//     pub(crate) iterator: I,
//     pub(crate) fold_op: F,
// }
//
// impl<R, I, F> ItemProducer for IteratorFold<I, F>
// where
//     R: Send,
//     I: ParallelIterator,
//     F: Fn(<I as Borrowed>::SeqIter) -> R,
// {
//     type Item = R;
// }

// we continue with the borrowed parallel type

pub struct BorrowedIteratorFold<'e, I, F: Sync> {
    iterator: I,
    fold_op: Dislocated<'e, F>,
}

impl<'e, R, I, F> ItemProducer for BorrowedIteratorFold<'e, I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<I as Borrowed>::SeqIter) -> R + Sync,
{
    type Item = R;
}

impl<'e, R, I, F> Divisible for BorrowedIteratorFold<'e, I, F>
where
    R: Send,
    I: ParallelIterator + Divisible,
    F: Fn(<I as Borrowed>::SeqIter) -> R + Sync,
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

impl<'extraction, 'e, R, I, F> Borrowed<'extraction> for BorrowedIteratorFold<'e, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I as Borrowed>::SeqIter) -> R + Sync,
{
    type ParIter = BorrowedIteratorFold<'e, <I as Borrowed<'extraction>>::ParIter, F>;
    type SeqIter = Once<R>;
}

impl<'e, R, I, F> ParallelIterator for BorrowedIteratorFold<'e, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I as Borrowed>::SeqIter) -> R + Sync,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as Borrowed<'extraction>>::ParIter {
        BorrowedIteratorFold {
            iterator: self.iterator.borrow_on_left_for(size),
            fold_op: self.fold_op,
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as Borrowed<'extraction>>::SeqIter {
        let i = self.iterator.sequential_borrow_on_left_for(size);
        let r = (self.fold_op)(i);
        once(r)
    }
}

impl<'e, R, I, F> FiniteParallelIterator for BorrowedIteratorFold<'e, I, F>
where
    R: Send,
    I: FiniteParallelIterator + Divisible,
    F: Fn(<I as Borrowed>::SeqIter) -> R + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
