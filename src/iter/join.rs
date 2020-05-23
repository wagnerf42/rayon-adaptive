use crate::prelude::*;

pub struct JoinPolicy<I> {
    pub(crate) base: I,
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
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        JoinPolicy {
            base: self.base.par_borrow(size),
            fallback: self.fallback,
        }
    }
}

impl<I> BorrowingParallelIterator for JoinPolicy<I>
where
    I: BorrowingParallelIterator,
{
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.base.seq_borrow(size)
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        //Now with the part completed function we don't need to fake the block sizes anymore
        Box::new(std::iter::repeat(self.fallback))
    }
    fn part_completed(&self) -> bool {
        //This is not very clear, maybe we don't want to mix the join_policy_size with the adaptive
        //scheduler. So to rephrase, we might be okay with giving the stealer an input of length
        //less than the fallback number.
        self.base.part_completed() || self.iterations_number() <= self.fallback
    }
}

impl<I: BorrowingParallelIterator> Divisible for JoinPolicy<I> {
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided() && self.base.iterations_number() > self.fallback
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            JoinPolicy {
                base: left,
                fallback: self.fallback,
            },
            JoinPolicy {
                base: right,
                fallback: self.fallback,
            },
        )
    }
}
