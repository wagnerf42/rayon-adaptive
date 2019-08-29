// map
use crate::prelude::*;

pub struct Map<I, F> {
    pub(crate) op: F,
    pub(crate) iterator: I,
}

impl<I, F> Divisible for Map<I, F>
where
    I: Divisible,
    F: Clone,
{
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            Map {
                op: self.op.clone(),
                iterator: left,
            },
            Map {
                op: self.op,
                iterator: right,
            },
        )
    }
}

impl<R, I, F> FiniteParallelIterator for Map<I, F>
where
    I: FiniteParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Clone + Send,
{
    type Iter = std::iter::Map<I::Iter, F>;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::Iter {
        self.iterator.to_sequential().map(self.op)
    }
}

impl<R, I, F> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(<I as ItemProducer>::Item) -> R + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        self.iterator.borrow_on_left_for(size).map(self.op.clone())
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        unimplemented!()
    }
}

impl<'extraction, R, I, F> FinitePart<'extraction> for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(<I as ItemProducer>::Item) -> R + Clone + Send,
{
    type ParIter = Map<<I as FinitePart<'extraction>>::ParIter, F>;
    type SeqIter = std::iter::Map<<I as FinitePart<'extraction>>::SeqIter, F>;
}

impl<R, I, F> ItemProducer for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(<I as ItemProducer>::Item) -> R,
{
    type Item = R;
}
