use crate::prelude::*;

pub struct JoinPolicy<I> {
    pub(crate) iterator: I,
    pub(crate) fallback: usize,
}

impl<I: FiniteParallelIterator + Divisible> Divisible for JoinPolicy<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible() && self.iterator.len() > self.fallback
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

impl<I: ParallelIterator> ItemProducer for JoinPolicy<I> {
    type Owner = JoinPolicy<I::Owner>;
    type Item = I::Item;
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> Borrowed<'e> for JoinPolicy<I> {
    type ParIter = JoinPolicy<<I::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter = <I::Owner as Borrowed<'e>>::SeqIter;
}

impl<I> ParallelIterator for JoinPolicy<I>
where
    I: ParallelIterator,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        JoinPolicy {
            iterator: self.iterator.borrow_on_left_for(size),
            fallback: self.fallback,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size)
    }
}

impl<I: FiniteParallelIterator> FiniteParallelIterator for JoinPolicy<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
