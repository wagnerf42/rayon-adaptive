use crate::prelude::*;

pub struct JoinPolicy<I> {
    pub(crate) iterator: I,
    pub(crate) fallback: usize,
}

impl<I: ItemProducer> ItemProducer for JoinPolicy<I> {
    type Item = I::Item;
}

impl<I: Powered> Powered for JoinPolicy<I> {
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> ParBorrowed<'e> for JoinPolicy<I> {
    type Iter = JoinPolicy<<I as ParBorrowed<'e>>::Iter>;
}

impl<'e, I: BorrowingParallelIterator> SeqBorrowed<'e> for JoinPolicy<I> {
    type Iter = <I as SeqBorrowed<'e>>::Iter;
}

impl<I> ParallelIterator for JoinPolicy<I>
where
    I: ParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.iterator.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        JoinPolicy {
            iterator: self.iterator.par_borrow(size),
            fallback: self.fallback,
        }
    }
}

impl<I> BorrowingParallelIterator for JoinPolicy<I>
where
    I: BorrowingParallelIterator,
{
    fn iterations_number(&self) -> usize {
        self.iterator.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.iterator.seq_borrow(size)
    }
}

impl<I: Divisible + BorrowingParallelIterator> Divisible for JoinPolicy<I> {
    fn should_be_divided(&self) -> bool {
        self.iterator.should_be_divided() && self.iterator.iterations_number() > self.fallback
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            JoinPolicy {
                iterator: left,
                fallback: self.fallback,
            },
            JoinPolicy {
                iterator: right,
                fallback: self.fallback,
            },
        )
    }
}
