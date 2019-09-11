use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct Filter<I, P> {
    pub(crate) iterator: I,
    pub(crate) filter_op: P,
}

pub struct BorrowingFilter<'a, I, P: Sync> {
    iterator: I,
    filter_op: Dislocated<'a, P>,
}

pub struct SeqFilter<'a, I, P: Sync> {
    iterator: I,
    filter_op: Dislocated<'a, P>,
}

impl<'a, I, P> Divisible for BorrowingFilter<'a, I, P>
where
    I: Divisible,
    P: Sync,
{
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            BorrowingFilter {
                iterator: left,
                filter_op: self.filter_op.clone(),
            },
            BorrowingFilter {
                iterator: right,
                filter_op: self.filter_op,
            },
        )
    }
}

impl<I, P> ItemProducer for Filter<I, P>
where
    I: ParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    type Owner = Filter<I::Owner, P>;
    type Item = I::Item;
}

impl<'a, I, P> ItemProducer for BorrowingFilter<'a, I, P>
where
    I: FiniteParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    type Owner = Filter<I::Owner, P>;
    type Item = I::Item;
}

impl<'e, I, P> Borrowed<'e> for Filter<I, P>
where
    I: ParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    type ParIter = BorrowingFilter<'e, <I::Owner as Borrowed<'e>>::ParIter, P>;
    type SeqIter = SeqFilter<'e, <I::Owner as Borrowed<'e>>::SeqIter, P>;
}

impl<I, P> ParallelIterator for Filter<I, P>
where
    I: ParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowingFilter {
            iterator: self.iterator.borrow_on_left_for(size),
            filter_op: Dislocated::new(&self.filter_op),
        }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        SeqFilter {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            filter_op: Dislocated::new(&self.filter_op),
        }
    }
}

impl<'a, I, P> ParallelIterator for BorrowingFilter<'a, I, P>
where
    I: FiniteParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowingFilter {
            iterator: self.iterator.borrow_on_left_for(size),
            filter_op: self.filter_op.clone(),
        }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        SeqFilter {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            filter_op: self.filter_op.clone(),
        }
    }
}

impl<'a, I, P> FiniteParallelIterator for BorrowingFilter<'a, I, P>
where
    I: FiniteParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

impl<I, P> FiniteParallelIterator for Filter<I, P>
where
    I: FiniteParallelIterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

impl<'a, I, P> Iterator for SeqFilter<'a, I, P>
where
    I: Iterator,
    P: Send + Sync + Fn(&I::Item) -> bool,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_candidate_item = self.iterator.next();
            if let Some(next_item) = next_candidate_item {
                if (self.filter_op)(&next_item) {
                    return Some(next_item);
                }
            } else {
                return None;
            }
        }
    }
}
