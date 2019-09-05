// map
use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct Map<I, F> {
    pub(crate) op: F,
    pub(crate) iterator: I,
}

impl<R, I, F> ItemProducer for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R,
{
    type Item = R;
}

impl<'extraction, R, I, F> FinitePart<'extraction> for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    type ParIter = BorrowingMap<'extraction, <I as FinitePart<'extraction>>::ParIter, F>;
    type SeqIter = SeqBorrowingMap<'extraction, <I as FinitePart<'extraction>>::SeqIter, F>;
}

impl<R, I, F> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        BorrowingMap {
            iterator: self.iterator.borrow_on_left_for(size),
            op: Dislocated::new(&self.op),
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        SeqBorrowingMap {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            op: Dislocated::new(&self.op),
        }
    }
}

impl<R, I, F> FiniteParallelIterator for Map<I, F>
where
    I: FiniteParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

pub struct BorrowingMap<'e, I, F: Sync> {
    op: Dislocated<'e, F>,
    iterator: I,
}

impl<'e, I, F> Divisible for BorrowingMap<'e, I, F>
where
    I: Divisible,
    F: Sync,
{
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            BorrowingMap {
                op: self.op,
                iterator: left,
            },
            BorrowingMap {
                op: self.op,
                iterator: right,
            },
        )
    }
}

impl<'e, R, I, F> ItemProducer for BorrowingMap<'e, I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Item = R;
}

impl<'e, 'extraction, R, I, F> FinitePart<'extraction> for BorrowingMap<'e, I, F>
where
    //'e: 'extraction,
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type ParIter = BorrowingMap<'e, <I as FinitePart<'extraction>>::ParIter, F>;
    type SeqIter = SeqBorrowingMap<'e, <I as FinitePart<'extraction>>::SeqIter, F>;
}

impl<'e, R, I, F> ParallelIterator for BorrowingMap<'e, I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Sync,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        BorrowingMap {
            iterator: self.iterator.borrow_on_left_for(size),
            op: self.op,
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        SeqBorrowingMap {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            op: self.op,
        }
    }
}

impl<'e, R, I, F> FiniteParallelIterator for BorrowingMap<'e, I, F>
where
    I: FiniteParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

pub struct SeqBorrowingMap<'e, I, F: Sync> {
    iterator: I,
    op: Dislocated<'e, F>,
}

impl<'e, R, I, F> Iterator for SeqBorrowingMap<'e, I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|e| (*self.op)(e))
    }
}
