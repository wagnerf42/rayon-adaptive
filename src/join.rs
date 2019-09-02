use crate::prelude::*;

pub struct JoinPolicy<I> {
    pub(crate) iterator: I,
    pub(crate) fallback: usize,
}

impl<I: FiniteParallelIterator> Divisible for JoinPolicy<I> {
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
    type Item = I::Item;
}

impl<'extraction, I: ParallelIterator> FinitePart<'extraction> for JoinPolicy<I> {
    type ParIter = JoinPolicy<<I as FinitePart<'extraction>>::ParIter>;
    type SeqIter = <I as FinitePart<'extraction>>::SeqIter;
}

impl<I> ParallelIterator for JoinPolicy<I>
where
    I: FiniteParallelIterator,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        JoinPolicy {
            iterator: self.iterator.borrow_on_left_for(size),
            fallback: self.fallback,
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size)
    }
}

impl<I: FiniteParallelIterator> FiniteParallelIterator for JoinPolicy<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
