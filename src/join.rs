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

impl<I: FiniteParallelIterator> FiniteParallelIterator for JoinPolicy<I> {
    type SequentialIterator = I::SequentialIterator;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential()
    }
}

impl<I> ParallelIterator for JoinPolicy<I>
where
    I: FiniteParallelIterator,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::Iter {
        JoinPolicy {
            iterator: self.iterator.borrow_on_left_for(size),
            fallback: self.fallback,
        }
    }
}

impl<'extraction, I: ParallelIterator> FinitePart<'extraction> for JoinPolicy<I> {
    type Iter = JoinPolicy<<I as FinitePart<'extraction>>::Iter>;
}

impl<I: ParallelIterator> ItemProducer for JoinPolicy<I> {
    type Item = I::Item;
}
