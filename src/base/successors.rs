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

// we use a trick to avoid all cloning.
// if owner_next is not None this is the one we use,
// else we use our next.
pub struct BorrowedSuccessors<'a, T: Sync, F: Sync, S: Sync> {
    count: usize,
    next: Option<T>,
    owner_next: Option<DislocatedMut<'a, T>>, // we might or might not have a pointer to the owner's next
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
            next: None,
            owner_next: Some(DislocatedMut::new(&mut self.next)),
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
            next: if let Some(on) = self.owner_next.as_mut() {
                on.borrow_mut()
            } else {
                DislocatedMut::new(self.next.as_mut().unwrap())
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
        if let Some(mut owner_next) = self.owner_next.take() {
            // ok, the owner_next is moving towards the right part
            let mut right_one = (self.skip)(&owner_next, left_count);
            std::mem::swap(&mut right_one, &mut owner_next);
            let right = BorrowedSuccessors {
                count: right_count,
                next: None,
                owner_next: Some(owner_next),
                succ: self.succ.clone(),
                skip: self.skip.clone(),
            };
            self.count = left_count;
            self.next = Some(right_one);
            (self, right)
        } else {
            // easy case
            let right_next = (self.skip)(self.next.as_ref().unwrap(), left_count);
            self.count = left_count;
            let right = BorrowedSuccessors {
                count: right_count,
                next: Some(right_next),
                owner_next: None,
                succ: self.succ.clone(),
                skip: self.skip.clone(),
            };
            (self, right)
        }
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
///                    |&i, n| i+(n as u64)).take(10_000_000).sum();
/// assert_eq!(s, 5_000_000 * 9_999_999)
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
