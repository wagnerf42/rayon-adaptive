use crate::prelude::*;
use std::iter::Take;

pub struct ParSuccessors<T, F, S> {
    pub(crate) next: T,
    pub(crate) succ: F,
    pub(crate) skip_op: S,
}

pub struct BoundedParSuccessors<'a, T, F, S> {
    next: T,
    remaining_iterations: usize,
    succ: F,
    skip_op: S,
    real_iterator_next: Option<&'a mut T>,
}

pub struct BorrowedSeqSuccessors<'a, T: Clone, F> {
    next: T,
    succ: F,
    real_iterator_next: &'a mut T,
}

impl<
        'extraction,
        T: 'static + Send + Clone, // this 'static saves the day
        F: Fn(T) -> T + Clone + Send,
        S: Send + Clone + Fn(T, usize) -> T,
    > FinitePart<'extraction> for ParSuccessors<T, F, S>
{
    type ParIter = BoundedParSuccessors<'extraction, T, F, S>;
    type SeqIter = Take<BorrowedSeqSuccessors<'extraction, T, F>>;
}

impl<
        'a,
        'extraction,
        T: 'static + Send + Clone,
        F: Fn(T) -> T + Clone + Send,
        S: Send + Clone + Fn(T, usize) -> T,
    > FinitePart<'extraction> for BoundedParSuccessors<'a, T, F, S>
{
    type ParIter = BoundedParSuccessors<'extraction, T, F, S>;
    type SeqIter = Take<BorrowedSeqSuccessors<'extraction, T, F>>;
}

impl<T, F, S> ItemProducer for ParSuccessors<T, F, S>
where
    T: Send,
{
    type Item = T;
}

impl<'a, T, F, S> ItemProducer for BoundedParSuccessors<'a, T, F, S>
where
    T: Send,
{
    type Item = T;
}

impl<T, F, S> ParallelIterator for ParSuccessors<T, F, S>
where
    T: Clone + 'static + Send,
    F: Fn(T) -> T + Clone + Send,
    S: Fn(T, usize) -> T + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        BoundedParSuccessors {
            next: self.next.clone(),
            remaining_iterations: size,
            succ: self.succ.clone(),
            skip_op: self.skip_op.clone(),
            real_iterator_next: Some(&mut self.next),
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        BorrowedSeqSuccessors {
            next: self.next.clone(),
            succ: self.succ.clone(),
            real_iterator_next: &mut self.next,
        }
        .take(size)
    }
}

impl<'a, T, F, S> ParallelIterator for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone + 'static + Send,
    F: Fn(T) -> T + Clone + Send,
    S: Fn(T, usize) -> T + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        BoundedParSuccessors {
            next: self.next.clone(),
            remaining_iterations: size,
            succ: self.succ.clone(),
            skip_op: self.skip_op.clone(),
            real_iterator_next: Some(&mut self.next),
        }
    }
    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        BorrowedSeqSuccessors {
            next: self.next.clone(),
            succ: self.succ.clone(),
            real_iterator_next: &mut self.next,
        }
        .take(size)
    }
}

impl<'a, T, F> Iterator for BorrowedSeqSuccessors<'a, T, F>
where
    T: Clone,
    F: Fn(T) -> T + Clone,
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
    T: Clone,
{
    fn drop(&mut self) {
        *self.real_iterator_next = self.next.clone()
    }
}

impl<'a, T, F, S> FiniteParallelIterator for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone + Send + 'static, //TODO: remove this static everywhere (with second lifetime ?)
    F: Fn(T) -> T + Clone + Send,
    S: Fn(T, usize) -> T + Clone + Send,
{
    fn len(&self) -> usize {
        self.remaining_iterations
    }
}

impl<'a, T, F, S> Divisible for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone,
    F: Fn(T) -> T + Clone,
    S: Fn(T, usize) -> T + Clone,
{
    fn is_divisible(&self) -> bool {
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
