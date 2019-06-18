//! `take` implementation.
use crate::prelude::*;
use crate::IndexedPower;
use crate::Policy;
use std::cmp::{max, min};

/// iterator adapter  for indexed iterateor used by 'take' function on `ParallelIterator`
pub struct Take<I> {
    pub(crate) iter: I,
    pub(crate) len: usize,
}

impl<I> Divisible for Take<I>
where
    I: ParallelIterator,
{
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        if self.iter.base_length() == None {
            // Infinite Iterator
            Some(self.len)
        } else {
            min(self.iter.base_length(), Some(self.len))
        }
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.divide_at(index);
        let left_len = min(index, self.len);
        let right_len = max(self.len - index, 0);
        (
            Take {
                iter: left,
                len: left_len,
            },
            Take {
                iter: right,
                len: right_len,
            },
        )
    }
}

impl<I> ParallelIterator for Take<I>
where
    I: ParallelIterator,
{
    type Item = I::Item;
    type SequentialIterator = std::iter::Take<I::SequentialIterator>;

    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        self.len -= size;
        self.iter.extract_iter(size).take(size)
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        self.iter.to_sequential().take(self.len)
    }

    fn policy(&self) -> Policy {
        self.iter.policy()
    }

    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iter.blocks_sizes()
    }
}
