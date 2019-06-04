use crate::divisibility::*;
use crate::prelude::*;
use crate::Policy;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
/// iterator adapter used by 'all' function on `ParallelIterator`
pub struct Interruptible<'a, I> {
    pub(crate) keepexec: &'a AtomicBool,
    pub(crate) iterator: I,
}

impl<'a, I> Divisible for Interruptible<'a, I>
where
    I: ParallelIterator,
{
    type Power = I::Power;
    fn base_length(&self) -> Option<usize> {
        if self.keepexec.load(Ordering::Relaxed) {
            self.iterator.base_length()
        } else {
            Some(0)
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iterator.divide_at(index);
        (
            Interruptible {
                keepexec: &self.keepexec,
                iterator: left,
            },
            Interruptible {
                keepexec: &self.keepexec,
                iterator: right,
            },
        )
    }
}

impl<'a, I> ParallelIterator for Interruptible<'a, I>
where
    I: ParallelIterator,
{
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
    fn extract_iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let (inner_iterator, remaining) = self.iterator.extract_iter(size);
        self.iterator = remaining;
        (inner_iterator, self)
    }

    fn policy(&self) -> Policy {
        self.iterator.policy()
    }

    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iterator.blocks_sizes()
    }
}
