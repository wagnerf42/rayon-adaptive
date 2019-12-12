//! so flatmap is a bit tricky.
//! it's because its power is a bit weird: between standard and basic.
//! we can't really borrow where we want since we don't know sizes.
//! we take the following approach :
//! - macro blocks sizes will be taken on the outer iterator
//! - micro blocks sizes will be taken on the inner iterators: no more than one iterator at a time

use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct FlatMap<I, F> {
    pub(crate) base: I,
    pub(crate) map_op: F,
}

impl<PI, I, F> ItemProducer for FlatMap<I, F>
where
    I: ParallelIterator,
    F: Fn(I::Item) -> PI,
    PI: IntoParallelIterator,
{
    type Item = PI::Item;
}

impl<I, F> Powered for FlatMap<I, F> {
    type Power = Standard;
}

impl<'e, PI, I, F> ParBorrowed<'e> for FlatMap<I, F>
where
    I: ParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    type Iter = BorrowingFlatMap<'e, PI, <I as ParBorrowed<'e>>::Iter, F>;
}

pub struct BorrowingFlatMap<'a, PI: IntoParallelIterator, I, F: Sync> {
    remaining_base: I,
    inner_iterator: <PI::Iter as ParBorrowed<'a>>::Iter,
    map_op: Dislocated<'a, F>,
}

impl<'a, PI, I, F> ItemProducer for BorrowingFlatMap<'a, PI, I, F>
where
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    type Item = PI::Item;
}

impl<'e, 'a, PI, I, F> SeqBorrowed<'e> for BorrowingFlatMap<'a, PI, I, F>
where
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    type Iter = <<PI::Iter as ParBorrowed<'a>>::Iter as SeqBorrowed<'e>>::Iter;
}

impl<PI, I, F> ParallelIterator for FlatMap<I, F>
where
    I: ParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        let outer = self.base.par_borrow(size);
        let inner = outer.seq_borrow(1).next().unwrap();
        let p_inner = (self.map_op)(inner).into_par_iter();
        let s = p_inner.bound_iterations_number(std::usize::MAX);
        let b_inner = p_inner.par_borrow(s);
        BorrowingFlatMap {
            remaining_base: outer,
            inner_iterator: b_inner,
            map_op: Dislocated::new(&self.map_op),
        }
    }
}

impl<'a, PI, I, F> BorrowingParallelIterator for BorrowingFlatMap<'a, PI, I, F>
where
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    type ScheduleType = I::ScheduleType;
    fn iterations_number(&self) -> usize {
        self.inner_iterator.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        //TODO: we need to do something on return :-(
        unimplemented!()
    }
}

impl<'a, PI, I, F> Divisible for BorrowingFlatMap<'a, PI, I, F>
where
    I: BorrowingParallelIterator,
    F: Fn(I::Item) -> PI + Sync,
    PI: IntoParallelIterator,
{
    fn should_be_divided(&self) -> bool {
        self.remaining_base.should_be_divided() || self.inner_iterator.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        if self.remaining_base.should_be_divided() {
            let (left, right) = self.remaining_base.divide();
            unimplemented!()
        } else {
            unimplemented!()
        }
    }
}
