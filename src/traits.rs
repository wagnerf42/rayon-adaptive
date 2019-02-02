//! This module contains all traits enabling us to express some parallelism.
use std;
use std::iter::{empty, Empty};
use std::ops::Range;
use std::ptr;

use crate::chunks::Chunks;
use crate::policy::ParametrizedInput;
use crate::Policy;

// markers for specialization
pub struct BasicPower();
pub struct BlockedPower();
pub struct IndexedPower();
pub struct BlockedOrMore();

pub trait Divisible: Sized + Send + Sync {
    type Power;
    /// Divide ourselves.
    /// Dividing shall never fail.
    fn divide(self) -> (Self, Self);
    /// Return our length.
    /// This length is used by the scheduler.
    /// Its meaning is :
    /// * give an approximate idea of the work required to process
    /// * a size of 0 means nothing left to do
    /// * TODO: for now we require base_length to be exactly equal to the number of loops
    /// we should remove this constraint
    fn base_length(&self) -> usize;
    fn with_policy(self, policy: Policy) -> ParametrizedInput<Self, Empty<usize>> {
        ParametrizedInput {
            input: self,
            policy,
            sizes: empty(),
        }
    }
}

pub trait DivisibleIntoBlocks: Divisible {
    /// Divide ourselves where requested.
    fn divide_at(self, index: usize) -> (Self, Self);
    /// Divide ourselves keeping right part in self.
    /// Returns the left part.
    /// NB: this is useful for iterators creation.
    fn cut_left_at(&mut self, index: usize) -> Self {
        // there is a lot of unsafe going on here.
        // I think it's ok. rust uses the same trick for moving iterators (vecs for example)
        unsafe {
            let my_copy = ptr::read(self);
            let (left, right) = my_copy.divide_at(index);
            let pointer_to_self = self as *mut Self;
            ptr::write(pointer_to_self, right);
            left
        }
    }
    /// Get a sequential iterator on chunks of Self of given sizes.
    fn chunks<S: Iterator<Item = usize>>(self, sizes: S) -> Chunks<Self, S> {
        Chunks {
            remaining: self,
            remaining_sizes: sizes,
        }
    }
}

pub trait DivisibleAtIndex: DivisibleIntoBlocks {}

impl<'a, T: Sync> Divisible for &'a [T] {
    type Power = IndexedPower;
    fn base_length(&self) -> usize {
        (*self as &[T]).len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: Sync> DivisibleIntoBlocks for &'a [T] {
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at(index)
    }
}

impl<'a, T: Sync> DivisibleAtIndex for &'a [T] {}

//TODO: I don't get why the compiler requires send here
impl<'a, T: 'a + Sync + Send> Divisible for &'a mut [T] {
    type Power = IndexedPower;
    fn base_length(&self) -> usize {
        (*self as &[T]).len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.base_length() / 2;
        self.split_at_mut(mid)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleIntoBlocks for &'a mut [T] {
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(index)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleAtIndex for &'a mut [T] {}

//TODO: be more generic but it seems complex
impl Divisible for Range<usize> {
    type Power = IndexedPower;
    fn base_length(&self) -> usize {
        self.len()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.start + ExactSizeIterator::len(&self) / 2;
        (self.start..mid, mid..self.end)
    }
}

//TODO: be more generic but it seems complex
impl DivisibleIntoBlocks for Range<usize> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        (
            self.start..(self.start + index),
            (self.start + index)..self.end,
        )
    }
}

impl DivisibleAtIndex for Range<usize> {}
