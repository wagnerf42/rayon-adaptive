//! Parallel `successors` iterator.
//! This is the only file in everything written so far where I could use iterators written
//! in the other direction.
//! When extracting sequential iterators we need the end user to complete the sequential iterator
//! BEFORE using the right parallel iterator.
//! Right now I don't see any way to ENFORCE this constraint by the compiler
//! (could we have extract_iter private ? does it make sense to have it private just for this file
//! ?)
//! Maybe it's not a big deal because I don't think end users would call this function.
use crate::prelude::*;
use crate::IndexedPower;

/// Parallel successors iterator (see `successors` fn).
pub struct ParSuccessors<T, F, S> {
    next: Option<T>,
    succ: F,
    skip_op: S,
    remaining_iterations: Option<usize>,
}

impl<T, F, S> Divisible for ParSuccessors<T, F, S>
where
    T: Clone,
    F: Fn(T) -> Option<T> + Clone,
    S: Fn(T, usize) -> Option<T> + Clone,
{
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        if self.next.is_none() {
            Some(0)
        } else {
            None
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let right_next = self.next.clone().and_then(|v| (self.skip_op)(v, index));
        (
            ParSuccessors {
                next: self.next,
                succ: self.succ.clone(),
                skip_op: self.skip_op.clone(),
                remaining_iterations: Some(index),
            },
            ParSuccessors {
                next: right_next,
                succ: self.succ.clone(),
                skip_op: self.skip_op.clone(),
                remaining_iterations: None,
            },
        )
    }
}

/// Sequential successors iterator obtained from the parallel one (see `successors` fn).
pub struct Successors<T, F> {
    next: Option<T>,
    succ: F,
    right_next: *mut Option<T>, // sadly we cannot borrow it because the associated type only has one lifetime
    remaining_iterations: Option<usize>,
}

impl<T, F, S> ParallelIterator for ParSuccessors<T, F, S>
where
    T: Clone + Send,
    F: Fn(T) -> Option<T> + Clone + Send,
    S: Fn(T, usize) -> Option<T> + Clone + Send,
{
    type Item = T;
    type SequentialIterator = Successors<T, F>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let next = self.next.take();
        Successors {
            next,
            succ: self.succ.clone(),
            right_next: &mut self.next as *mut Option<T>,
            remaining_iterations: Some(size),
        }
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        Successors {
            next: self.next,
            succ: self.succ,
            right_next: std::ptr::null_mut(),
            remaining_iterations: self.remaining_iterations,
        }
    }
}

impl<T, F> Iterator for Successors<T, F>
where
    F: Fn(T) -> Option<T>,
    T: Clone,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next.is_none() || self.remaining_iterations.unwrap_or(1) == 0 {
            // nothing left to do
            None
        } else {
            let returned_value = self.next.clone();
            // ok let's advance
            let s = (self.succ)(self.next.take().unwrap());
            self.remaining_iterations = self.remaining_iterations.map(|i| i - 1);
            if self.remaining_iterations.unwrap_or(1) == 0 {
                // if this is the last iteration we need to propagate the value
                // to the other side (if any).
                if !self.right_next.is_null() {
                    unsafe { self.right_next.write(s.clone()) }
                }
            }
            self.next = s;
            returned_value
        }
    }
}

/// This function returns a `ParallelIterator` on successors (it is the parallel version
/// of the corresponding sequential function from `std::iter::successors`).
/// The only way to make it work with performances is if we have a way to shortcut to the nth
/// element.
/// There is a nice application of this function for random number generators.
///
/// Here is a small example where the successor is just adding one.
/// The shorcut function adds directly n.
///
/// # Example:
///
/// ```
/// use rayon_adaptive::prelude::*;
/// use rayon_adaptive::successors;
/// let mut v: Vec<usize> = vec![0; 100_000];
/// v.as_mut_slice()
///     .into_par_iter()
///     .zip(successors(Some(0), |i| Some(i + 1), |i, s| Some(i + s)))
///     .for_each(|(r, i)| *r = i);
/// let expected: Vec<usize> = (0..100_000).collect();
/// assert_eq!(v, expected);
/// ```
pub fn successors<T, F, S>(first: Option<T>, succ: F, skip_op: S) -> ParSuccessors<T, F, S>
where
    T: Clone,
    F: Fn(T) -> Option<T> + Clone,
    S: Fn(T, usize) -> Option<T> + Clone,
{
    ParSuccessors {
        next: first,
        succ,
        skip_op,
        remaining_iterations: None,
    }
}
