use crate::prelude::*;

/// Parallel iterator which may be of two different types.
/// This is simplifying the code in other places like flatmap or chain.
pub enum EitherIter<I, J> {
    I(I),
    J(J),
}

impl<I, J> ItemProducer for EitherIter<I, J>
where
    I: ItemProducer,
    J: ItemProducer<Item = I::Item>,
{
    type Item = I::Item;
}

impl<I, J> Powered for EitherIter<I, J>
where
    I: Powered,
    J: Powered + MinPower<I::Power>,
{
    type Power = <J as MinPower<I::Power>>::Min;
}

impl<'e, I, J> ParBorrowed<'e> for EitherIter<I, J>
where
    I: ParallelIterator,
    J: ParallelIterator<Item = I::Item> + MinPower<I::Power>,
{
    type Iter = EitherIter<<I as ParBorrowed<'e>>::Iter, <J as ParBorrowed<'e>>::Iter>;
}

impl<'e, I, J> SeqBorrowed<'e> for EitherIter<I, J>
where
    I: BorrowingParallelIterator,
    J: BorrowingParallelIterator<Item = I::Item>,
{
    type Iter = EitherSeqIter<<I as SeqBorrowed<'e>>::Iter, <J as SeqBorrowed<'e>>::Iter>;
}

impl<I, J> BorrowingParallelIterator for EitherIter<I, J>
where
    I: BorrowingParallelIterator,
    J: BorrowingParallelIterator<Item = I::Item>,
{
    type ScheduleType = I::ScheduleType;
    fn iterations_number(&self) -> usize {
        match self {
            EitherIter::I(i) => i.iterations_number(),
            EitherIter::J(j) => j.iterations_number(),
        }
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        match self {
            EitherIter::I(i) => EitherSeqIter::I(i.seq_borrow(size)),
            EitherIter::J(j) => EitherSeqIter::J(j.seq_borrow(size)),
        }
    }
}

impl<I, J> ParallelIterator for EitherIter<I, J>
where
    I: ParallelIterator,
    J: ParallelIterator<Item = I::Item> + MinPower<I::Power>,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        match self {
            EitherIter::I(i) => i.bound_iterations_number(size),
            EitherIter::J(j) => j.bound_iterations_number(size),
        }
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        match self {
            EitherIter::I(i) => EitherIter::I(i.par_borrow(size)),
            EitherIter::J(j) => EitherIter::J(j.par_borrow(size)),
        }
    }
}

impl<I, J> Divisible for EitherIter<I, J>
where
    I: BorrowingParallelIterator,
    J: BorrowingParallelIterator<Item = I::Item>,
{
    fn should_be_divided(&self) -> bool {
        match self {
            EitherIter::I(i) => i.should_be_divided(),
            EitherIter::J(j) => j.should_be_divided(),
        }
    }
    fn divide(self) -> (Self, Self) {
        match self {
            EitherIter::I(i) => {
                let (left, right) = i.divide();
                (EitherIter::I(left), EitherIter::I(right))
            }
            EitherIter::J(j) => {
                let (left, right) = j.divide();
                (EitherIter::J(left), EitherIter::J(right))
            }
        }
    }
}

// Sequential Iterator.

pub enum EitherSeqIter<I, J> {
    I(I),
    J(J),
}

impl<I, J> Iterator for EitherSeqIter<I, J>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherSeqIter::I(i) => i.next(),
            EitherSeqIter::J(j) => j.next(),
        }
    }
}
