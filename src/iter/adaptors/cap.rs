//! Cap algorithm to the given number of threads.
//! TODO: this does not work nicely with blocks for now.
//! we need to make a distinction between dividing on the left for sequential iterations
//! and dividing on the right for parallel iterations.
use crate::prelude::*;
use crate::Policy;
use std::ops::Drop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Switch underlying iterator to adaptive policy (if not specified yet)
/// and cap the number of threads for it's tasks to the given number.
pub struct Cap<I> {
    pub(crate) iterator: I,
    pub(crate) count: Arc<AtomicUsize>,
    pub(crate) limit: usize,
}

impl<I: ParallelIterator> Divisible for Cap<I> {
    type Power = I::Power;
    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }
    // we specialize the division so that we avoid decrementing the counter for the sequential
    // blocks
    fn divide_on_left_at(&mut self, index: usize) -> (Self) {
        let left = self.iterator.divide_on_left_at(index);
        Cap {
            iterator: left,
            count: self.count.clone(),
            limit: self.limit,
        }
    }
    fn divide_at(mut self, mut index: usize) -> (Self, Self) {
        let usage = self.count.fetch_add(1, Ordering::Relaxed);
        let authorize_division = self.base_length().is_none() || usage < self.limit - 1;
        if !authorize_division {
            // let's cut a right piece of size 0
            index = self.base_length().expect("infinite iterator");
            self.count.fetch_sub(1, Ordering::Relaxed); // decrement immediately by one because we will not create the iterator
        }
        let (left, right) = self.iterator.divide_at(index);
        self.iterator = left;
        let count = self.count.clone();
        let limit = self.limit;
        (
            self,
            Cap {
                iterator: right,
                count,
                limit,
            },
        )
    }
}

/// Sequential iterator for `Cap`.
pub struct CapSeq<I> {
    iterator: I,
    count: Option<Arc<AtomicUsize>>,
}

impl<I> Drop for CapSeq<I> {
    fn drop(&mut self) {
        if let Some(count) = &self.count {
            count.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

impl<I: Iterator> Iterator for CapSeq<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

impl<I: ParallelIterator> ParallelIterator for Cap<I> {
    type Item = I::Item;
    type SequentialIterator = CapSeq<I::SequentialIterator>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let iterator = self.iterator.extract_iter(size);
        CapSeq {
            iterator,
            count: None,
        }
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        CapSeq {
            iterator: self.iterator.to_sequential(),
            count: Some(self.count),
        }
    }
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iterator.blocks_sizes()
    }
    fn policy(&self) -> Policy {
        match self.iterator.policy() {
            Policy::Adaptive(min, max) => Policy::Adaptive(min, max),
            _ => unimplemented!("set a default policy for cap"),
        }
    }
}
