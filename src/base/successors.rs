use crate::dislocated::{Dislocated, DislocatedMut};
use crate::prelude::*;
use std::iter::Take;

pub struct ParSuccessors<T, F, S> {
    pub(crate) next: T,
    pub(crate) succ: F,
    pub(crate) skip_op: S,
}

pub struct BoundedParSuccessors<'a, T: Sync, F: Sync, S: Sync> {
    next: T,
    remaining_iterations: usize,
    succ: Dislocated<'a, F>,
    skip_op: Dislocated<'a, S>,
    real_iterator_next: Option<DislocatedMut<'a, T>>,
}

pub struct BorrowedSeqSuccessors<'a, T: Clone + Sync, F: Fn(T) -> T + Sync> {
    next: T,
    succ: Dislocated<'a, F>,
    real_iterator_next: DislocatedMut<'a, T>,
}

impl<T, F, S> ItemProducer for ParSuccessors<T, F, S>
where
    T: Send + Sync,
{
    type Item = T;
}

impl<T, F, S> Powered for ParSuccessors<T, F, S> {
    type Power = Indexed;
}

impl<'e, T, F, S> ParBorrowed<'e> for ParSuccessors<T, F, S>
where
    T: Send + Sync + Clone,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    type Iter = BoundedParSuccessors<'e, T, F, S>;
}

impl<'a, T, F, S> ItemProducer for BoundedParSuccessors<'a, T, F, S>
where
    T: Send + Sync,
    F: Sync,
    S: Sync,
{
    type Item = T;
}

impl<'e, 'a, T, F, S> SeqBorrowed<'e> for BoundedParSuccessors<'a, T, F, S>
where
    T: Send + Sync + Clone,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    type Iter = Take<BorrowedSeqSuccessors<'e, T, F>>;
}

impl<T, F, S> ParallelIterator for ParSuccessors<T, F, S>
where
    T: Send + Sync + Clone,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BoundedParSuccessors {
            next: self.next.clone(),
            remaining_iterations: size,
            succ: Dislocated::new(&self.succ),
            skip_op: Dislocated::new(&self.skip_op),
            real_iterator_next: Some(DislocatedMut::new(&mut self.next)),
        }
    }
}

impl<'a, T, F, S> BorrowingParallelIterator for BoundedParSuccessors<'a, T, F, S>
where
    T: Send + Sync + Clone,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        BorrowedSeqSuccessors {
            next: self.next.clone(),
            succ: Dislocated::new(&self.succ),
            real_iterator_next: DislocatedMut::new(&mut self.next),
        }
        .take(size)
    }
    fn iterations_number(&self) -> usize {
        self.remaining_iterations
    }
}

impl<'a, T, F> Iterator for BorrowedSeqSuccessors<'a, T, F>
where
    T: Send + Sync + Clone,
    F: Fn(T) -> T + Send + Sync,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let next_next = (self.succ)(self.next.clone());
        let current_next = std::mem::replace(&mut self.next, next_next);
        Some(current_next)
    }
}

impl<'a, T, F> Drop for BorrowedSeqSuccessors<'a, T, F>
where
    T: Clone + Sync,
    F: Fn(T) -> T + Sync,
{
    fn drop(&mut self) {
        *self.real_iterator_next = self.next.clone()
    }
}

impl<'a, T, F, S> Divisible for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone + Sync,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    fn should_be_divided(&self) -> bool {
        self.remaining_iterations > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mid = self.remaining_iterations / 2;
        let right_next = (self.skip_op)(self.next.clone(), mid);
        let right = BoundedParSuccessors {
            next: right_next,
            remaining_iterations: self.remaining_iterations - mid,
            succ: self.succ.clone(),
            skip_op: self.skip_op.clone(),
            real_iterator_next: self.real_iterator_next.take(),
        };
        self.remaining_iterations = mid;
        (self, right)
    }
}

pub fn successors<'a, T, F, S>(next: T, succ: F, skip_op: S) -> ParSuccessors<T, F, S>
where
    T: Clone + Sync,
    F: Fn(T) -> T + Send + Sync,
    S: Fn(T, usize) -> T + Send + Sync,
{
    ParSuccessors {
        next,
        succ,
        skip_op,
    }
}
