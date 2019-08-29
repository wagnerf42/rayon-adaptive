// map
use crate::prelude::*;
use std::marker::PhantomData;

struct Map<I, E, F> {
    op: F,
    iterator: E,
    phantom: PhantomData<I>,
}

impl<I, E, F> Divisible for Map<I, E, F>
where
    E: Divisible,
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
                phantom: PhantomData,
            },
            Map {
                op: self.op,
                iterator: right,
                phantom: PhantomData,
            },
        )
    }
}

impl<R, I, E, F> ParallelIterator for Map<I, E, F>
where
    I: Send,
    E: ParallelIterator,
    R: Send,
    F: Fn(E::Item) -> R + Clone + Send,
{
    type Item = R;
    type SequentialIterator = std::iter::Map<E::SequentialIterator, F>;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential().map(self.op)
    }
}

impl<R, E, F, I> Extractible<R> for Map<I, E, F>
where
    I: Send,
    E: Extractible<I>,
    R: Send,
    F: Fn(I) -> R + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as ExtractiblePart<'extraction, R>>::BorrowedPart {
        self.iterator.borrow_on_left_for(size).map(&self.op)
    }
}

impl<'extraction, R, E, F, I> ExtractiblePart<'extraction, R> for Map<I, E, F>
where
    I: Send,
    E: Extractible<I>,
    R: Send,
    F: Fn(I) -> R + Clone + Send,
{
    type BorrowedPart = Map<I, <E as ExtractiblePart<'extraction, I>>::BorrowedPart, F>;
}
