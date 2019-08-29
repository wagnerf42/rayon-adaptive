// map
use crate::prelude::*;

pub struct Map<I, F> {
    op: F,
    iterator: I,
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

impl<R, I, F> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Clone + Send,
{
    type Item = R;
    type SequentialIterator = std::iter::Map<I::SequentialIterator, F>;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential().map(self.op)
    }
}

impl<R, I, F> Extractible for Map<I, F>
where
    I: Extractible,
    R: Send,
    F: Fn(<I as ExtractibleItem>::Item) -> R + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as ExtractiblePart<'extraction>>::BorrowedPart {
        self.iterator.borrow_on_left_for(size).map(&self.op)
    }
}

impl<'extraction, R, I, F> ExtractiblePart<'extraction> for Map<I, F>
where
    I: Extractible,
    R: Send,
    F: Fn(<I as ExtractibleItem>::Item) -> R + Clone + Send,
{
    type BorrowedPart = Map<<I as ExtractiblePart<'extraction>>::BorrowedPart, F>;
}

impl<R, I, F> ExtractibleItem for Map<I, F>
where
    F: Fn(<I as ExtractibleItem>::Item) -> R,
{
    type Item = R;
}
