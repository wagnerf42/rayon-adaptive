//! This module contains all traits enabling us to express some parallelism.
use std;

/// Some work we could schedule adaptively.
pub struct AdaptiveWork<D: Divisible, O: Output, W: Fn(D) -> (Option<D>, O)> {
    input: D,
    work_function: W,
}

//TODO: add a `schedule` method to the adaptive work
//TODO: remove Block

pub trait Divisible: Sized {
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Return our length.
    fn len(&self) -> usize;
    /// Register the work function and get back some `AdaptiveWork` ready for scheduling.
    fn work<O: Output, W: Fn(Self) -> (Option<Self>, O)>(
        self,
        work_function: W,
    ) -> AdaptiveWork<Self, O, W> {
        AdaptiveWork {
            input: self,
            work_function,
        }
    }
}

/// All inputs should implement this trait.
pub trait Block: Divisible {
    type Output: Output;
    /// Compute some output for this block. Return what's left to do if any and result.
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output);
}

/// All outputs should implement this trait.
pub trait Output: Sized {
    /// Merge two outputs into one.
    fn fuse(self, other: Self) -> Self;
}

impl Output for () {
    fn fuse(self, _other: Self) -> Self {
        ()
    }
}

impl<'a, T> Divisible for &'a [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: 'a> Divisible for &'a mut [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at_mut(mid)
    }
}

impl<A: Divisible, B: Divisible> Divisible for (A, B) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), self.1.len())
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        ((left_a, left_b), (right_a, right_b))
    }
}

impl<A: Divisible, B: Divisible, C: Divisible> Divisible for (A, B, C) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), std::cmp::min(self.1.len(), self.2.len()))
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        let (left_c, right_c) = self.2.split();
        ((left_a, left_b, left_c), (right_a, right_b, right_c))
    }
}
