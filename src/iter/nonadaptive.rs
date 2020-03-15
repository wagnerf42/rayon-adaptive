use crate::prelude::*;
pub struct NonAdaptiveIter<I> {
    pub(crate) base: I,
}

impl<I: ItemProducer> ItemProducer for NonAdaptiveIter<I> {
    type Item = I::Item;
}

impl<I: Powered> Powered for NonAdaptiveIter<I> {
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> ParBorrowed<'e> for NonAdaptiveIter<I> {
    type Iter = NonAdaptiveIter<<I as ParBorrowed<'e>>::Iter>;
}

impl<'e, I: BorrowingParallelIterator> SeqBorrowed<'e> for NonAdaptiveIter<I> {
    type Iter = <I as SeqBorrowed<'e>>::Iter;
}

impl<I> ParallelIterator for NonAdaptiveIter<I>
where
    I: ParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        NonAdaptiveIter {
            base: self.base.par_borrow(size),
        }
    }
}

impl<I> BorrowingParallelIterator for NonAdaptiveIter<I>
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
        Box::new(std::iter::repeat(std::usize::MAX))
    }
    fn part_completed(&self) -> bool {
        //This adaptor will hence never allow a steal
        true
    }
}

impl<I: BorrowingParallelIterator> Divisible for NonAdaptiveIter<I> {
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            NonAdaptiveIter { base: left },
            NonAdaptiveIter { base: right },
        )
    }
}
