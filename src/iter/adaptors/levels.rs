//! Levels iterator.
use crate::prelude::*;

use crate::Policy;

/// Levels iterator adapter, returned by `levels` function on `ParallelIterator`
pub struct Levels<I> {
    pub(crate) iter: I,
    pub(crate) levels: usize,
}

impl<I: ParallelIterator> Divisible for Levels<I> {
    type Power = I::Power;

    fn base_length(&self) -> Option<usize> {
        if self.levels > 0 {
            self.iter.base_length()
        } else {
            Some(std::cmp::min(
                self.iter
                    .base_length()
                    .expect("levels does not work on infinite iterators"),
                1,
            ))
        }
    }

    fn divide_on_left_at(&mut self, index: usize) -> Self {
        let left = self.iter.divide_on_left_at(index);
        Levels {
            iter: left,
            levels: self.levels,
        }
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.divide_at(index);
        let levels = if self.levels == 0 { 0 } else { self.levels - 1 };
        (
            Levels { iter: left, levels },
            Levels {
                iter: right,
                levels,
            },
        )
    }
}

impl<I: ParallelIterator> ParallelIterator for Levels<I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;

    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        self.iter.extract_iter(size)
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        self.iter.to_sequential()
    }

    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iter.blocks_sizes()
    }

    fn policy(&self) -> Policy {
        self.iter.policy()
    }
}

// (0..100_000).cut().with_policy(Policy::Join(1000)).levels(3).map(|_| 1).sum() == 8
