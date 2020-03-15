//! Struct for simple fold op on sequential iterators.
use crate::dislocated::Dislocated;
use crate::prelude::*;
use std::iter::{once, Once};

// we start with the parallel type

/// Parallel Iterator where all sequential iterators are folded by given function (see
/// `iterator_fold` method).
pub struct IteratorFold<I, F> {
    pub(crate) base: I,
    pub(crate) fold_op: F,
}

impl<R, I, F> ItemProducer for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<<I as ParBorrowed>::Iter as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    type Item = R;
}

impl<I, F> Powered for IteratorFold<I, F>
where
    I: ParallelIterator,
{
    type Power = I::Power;
}

impl<'e, R, I, F> ParBorrowed<'e> for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<<I as ParBorrowed>::Iter as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    type Iter = BorrowedIteratorFold<'e, <I as ParBorrowed<'e>>::Iter, F>;
}

impl<R, I, F> ParallelIterator for IteratorFold<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(<<I as ParBorrowed>::Iter as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowedIteratorFold {
            base: self.base.par_borrow(size),
            fold_op: Dislocated::new(&self.fold_op),
        }
    }
}

// we continue with the borrowed parallel type

pub struct BorrowedIteratorFold<'e, I, F: Sync> {
    base: I,
    fold_op: Dislocated<'e, F>,
}

impl<'a, R, I, F> ItemProducer for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(<I as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    type Item = R;
}

impl<'e, 'a, R, I, F> SeqBorrowed<'e> for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(<I as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    type Iter = Once<R>;
}

impl<'a, R, I, F> BorrowingParallelIterator for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(<I as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        let i = self.base.seq_borrow(size);
        let r = (self.fold_op)(i);
        once(r)
    }
    fn part_completed(&self) -> bool {
        self.base.part_completed()
    }
}

impl<'a, R, I, F> Divisible for BorrowedIteratorFold<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(<I as SeqBorrowed>::Iter) -> R + Send + Sync,
{
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            BorrowedIteratorFold {
                base: left,
                fold_op: self.fold_op,
            },
            BorrowedIteratorFold {
                base: right,
                fold_op: self.fold_op,
            },
        )
    }
}
