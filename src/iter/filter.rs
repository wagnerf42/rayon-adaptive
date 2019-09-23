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
    fn should_be_divided(&self) -> bool {
        self.iterator.should_be_divided()
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
    I: ItemProducer,
{
    type Item = I::Item;
}

impl<I, P> Powered for Filter<I, P>
where
    I: Powered,
{
    type Power = I::Power;
}

impl<'a, I, P> ItemProducer for BorrowingFilter<'a, I, P>
where
    I: ItemProducer,
    P: Sync,
{
    type Item = I::Item;
}

impl<'e, I, P> ParBorrowed<'e> for Filter<I, P>
where
    I: ParallelIterator,
    P: Sync + Fn(&I::Item) -> bool,
{
    type Iter = BorrowingFilter<'e, <I as ParBorrowed<'e>>::Iter, P>;
}

impl<I, P> ParallelIterator for Filter<I, P>
where
    I: ParallelIterator,
    P: Sync + Fn(&I::Item) -> bool,
{
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowingFilter {
            iterator: self.iterator.par_borrow(size),
            filter_op: Dislocated::new(&self.filter_op),
        }
    }
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.iterator.bound_iterations_number(size)
    }
}

impl<'e, 'a, I, P> SeqBorrowed<'e> for BorrowingFilter<'a, I, P>
where
    I: BorrowingParallelIterator,
    P: Sync + Fn(&I::Item) -> bool,
{
    type Iter = SeqFilter<'e, <I as SeqBorrowed<'e>>::Iter, P>;
}

impl<'a, I, P> BorrowingParallelIterator for BorrowingFilter<'a, I, P>
where
    I: BorrowingParallelIterator,
    P: Sync + Fn(&I::Item) -> bool,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        SeqFilter {
            iterator: self.iterator.seq_borrow(size),
            filter_op: self.filter_op.clone(),
        }
    }
    fn iterations_number(&self) -> usize {
        self.iterator.iterations_number()
    }
}

impl<'a, I, P> Iterator for SeqFilter<'a, I, P>
where
    I: Iterator,
    P: Sync + Fn(&I::Item) -> bool,
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
