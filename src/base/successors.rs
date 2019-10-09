//! Implementation of a parallel version of `std::iter::successors`.
//!
//! One choice has been made here which makes things complex :
//! we do not require T: Clone
//! this implies you need to handle next through references
//!
//! On the other side we ditched Options in `succ`.
use crate::dislocated::{Dislocated, DislocatedMut};
use crate::prelude::*;
use std::iter::Take;

pub struct Successors<T, F, S> {
    next: T,
    succ: F,
    skip: S,
}

pub struct BorrowedSuccessors<'a, T: Sync, F: Sync, S: Sync> {
    count: usize,
    next: either::Either<T, DislocatedMut<'a, T>>, // after division we might own it
    succ: Dislocated<'a, F>,
    skip: Dislocated<'a, S>,
}

pub struct SeqSuccessors<'a, T: Sync, F: Sync> {
    next: DislocatedMut<'a, T>, // we always have a pointer to the borrowing iter's next
    succ: Dislocated<'a, F>,
}

impl<T, F, S> ItemProducer for Successors<T, F, S>
where
    T: Sync + Send,
    F: Sync,
    S: Sync,
{
    type Item = T;
}

impl<T, F, S> Powered for Successors<T, F, S>
where
    T: Sync + Send,
    F: Sync,
    S: Sync,
{
    type Power = Indexed;
}

impl<'a, T, F, S> ItemProducer for BorrowedSuccessors<'a, T, F, S>
where
    T: Sync + Send,
    F: Sync,
    S: Sync,
{
    type Item = T;
}

impl<'e, T, F, S> ParBorrowed<'e> for Successors<T, F, S>
where
    T: Sync + Send,
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    type Iter = BorrowedSuccessors<'e, T, F, S>;
}

impl<'a, 'e, T, F, S> SeqBorrowed<'e> for BorrowedSuccessors<'a, T, F, S>
where
    T: Sync + Send,
    F: Fn(&T) -> T + Sync,
    S: Sync,
{
    type Iter = Take<SeqSuccessors<'e, T, F>>;
}

impl<T, F, S> ParallelIterator for Successors<T, F, S>
where
    T: Sync + Send,
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        size
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowedSuccessors {
            count: size,
            next: either::Right(DislocatedMut::new(&mut self.next)),
            succ: Dislocated::new(&self.succ),
            skip: Dislocated::new(&self.skip),
        }
    }
}

impl<'a, T, F, S> BorrowingParallelIterator for BorrowedSuccessors<'a, T, F, S>
where
    T: Sync + Send,
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    fn iterations_number(&self) -> usize {
        self.count
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.count -= size;
        SeqSuccessors {
            next: match &mut self.next {
                either::Either::Left(n) => DislocatedMut::new(n),
                either::Either::Right(n) => n.borrow_mut(),
            },
            succ: self.succ.clone(),
        }
        .take(size)
    }
}

impl<'a, T, F> Iterator for SeqSuccessors<'a, T, F>
where
    T: Sync,
    F: Fn(&T) -> T + Sync,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut next_one = (self.succ)(&self.next);
        std::mem::swap(&mut next_one, &mut self.next);
        Some(next_one)
    }
}

impl<'a, T, F, S> Divisible for BorrowedSuccessors<'a, T, F, S>
where
    T: Sync + Send,
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    fn should_be_divided(&self) -> bool {
        self.count > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let left_count = self.count / 2;
        let right_count = self.count - left_count;
        let (left_next, right_next) = match self.next {
            either::Either::Left(n) => {
                let right_next = (self.skip)(&n, left_count);
                (either::Either::Left(n), either::Either::Left(right_next))
            }
            either::Either::Right(mut n) => {
                let mut right_next = (self.skip)(&n, left_count);
                std::mem::swap(&mut right_next, &mut n); // put it in owner's next
                (either::Either::Left(right_next), either::Either::Right(n))
            }
        };
        let right = BorrowedSuccessors {
            count: right_count,
            next: right_next,
            succ: self.succ.clone(),
            skip: self.skip.clone(),
        };
        self.count = left_count;
        self.next = left_next;
        (self, right)
    }
}

/// Potentially infinite iterator on successors.
/// You need to provide a fast way to `skip` elements
///
/// # Example
///
/// ```
/// use rayon_adaptive::prelude::*;
/// use rayon_adaptive::successors;
/// // let's fake a range just for testing
/// let s:u64 = successors(0u64,
///                    |&i| i+1,
///                    |&i, n| i+(n as u64)).take(10_000).sum();
/// assert_eq!(s, 5_000* 9_999)
/// ```
pub fn successors<T, F, S>(first: T, succ: F, skip: S) -> Successors<T, F, S>
where
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    Successors {
        next: first,
        succ,
        skip,
    }
}
