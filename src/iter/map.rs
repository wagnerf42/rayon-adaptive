// map
use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct Map<I, F> {
    pub(crate) op: F,
    pub(crate) base: I,
}

impl<R, I, F> ItemProducer for Map<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Item = R;
}

impl<R, I, F> Powered for Map<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Power = I::Power;
}

impl<'e, R, I, F> ParBorrowed<'e> for Map<I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Iter = BorrowingMap<'e, <I as ParBorrowed<'e>>::Iter, F>;
}

impl<R, I, F> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowingMap {
            base: self.base.par_borrow(size),
            op: Dislocated::new(&self.op),
        }
    }
}

pub struct BorrowingMap<'a, I, F: Sync> {
    op: Dislocated<'a, F>,
    base: I,
}

impl<'a, I, F> Divisible for BorrowingMap<'a, I, F>
where
    I: Divisible,
    F: Sync,
{
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            BorrowingMap {
                op: self.op,
                base: left,
            },
            BorrowingMap {
                op: self.op,
                base: right,
            },
        )
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.divide_at(index);
        (
            BorrowingMap {
                op: self.op,
                base: left,
            },
            BorrowingMap {
                op: self.op,
                base: right,
            },
        )
    }
}

impl<'a, R, I, F> ItemProducer for BorrowingMap<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Item = R;
}

impl<'e, 'a, R, I, F> SeqBorrowed<'e> for BorrowingMap<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Iter = SeqBorrowingMap<'e, <I as SeqBorrowed<'e>>::Iter, F>;
}

impl<'a, R, I, F> BorrowingParallelIterator for BorrowingMap<'a, I, F>
where
    R: Send,
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> R + Sync,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        SeqBorrowingMap {
            iterator: self.base.seq_borrow(size),
            op: self.op,
        }
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        self.base.micro_blocks_sizes()
    }
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
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
