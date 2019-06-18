//! Implement `dedup` in an adaptive way.
//! This is an interesting example because the extract iter is at lower overhead than the division
//! (well slightly).
use crate::prelude::*;
use crate::IndexedPower;
use std::cmp::PartialEq;
use std::iter::Chain;

/// Parallel Dedup Iterator
pub struct Dedup<I: ParallelIterator> {
    pub(crate) first: Option<I::Item>,
    pub(crate) iter: I,
    pub(crate) last: Option<I::Item>,
}

/// Sequential Dedup iterator obtained from the parallel one (see `dedup` fn).
pub struct DedupSeq<I: Iterator> {
    /// Elements before deduplicating.
    /// The chain allows us to handle the cloned element.
    iter: Chain<I, std::option::IntoIter<I::Item>>,
    /// Raw pointer to dispatch last result to the remaining parallel iterator.
    right_first: *mut Option<I::Item>,
    /// Previous element we compare to.
    previous_item: Option<I::Item>,
}

impl<I> Divisible for Dedup<I>
where
    I::Item: Clone + PartialEq,
    I: ParallelIterator,
{
    type Power = IndexedPower;

    fn base_length(&self) -> Option<usize> {
        self.iter
            .base_length()
            .map(|l| l + self.last.is_some() as usize)
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.divide_at(index);
        // we clone the last element of the left part to put it on both sides
        // as previous item to compare to on the right side
        // and as the last item to iterate on (possibly) on the left side.
        let (all_left_but_last, last_left) = left.divide_at(index - 1);
        let last = last_left.to_sequential().next();
        let first_right = last.clone();

        (
            Dedup {
                first: self.first,
                iter: all_left_but_last,
                last,
            },
            Dedup {
                first: first_right,
                iter: right,
                last: self.last,
            },
        )
    }
}

impl<I> ParallelIterator for Dedup<I>
where
    I::Item: Clone + PartialEq + Copy,
    I: ParallelIterator,
{
    type Item = I::Item;
    type SequentialIterator = DedupSeq<I::SequentialIterator>;

    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let duplicated_iter = self.iter.extract_iter(size).chain(None);

        DedupSeq {
            iter: duplicated_iter,
            right_first: &mut self.first as *mut Option<I::Item>,
            previous_item: self.first,
        }
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        let duplicated_iter = self.iter.to_sequential().chain(self.last);
        DedupSeq {
            iter: duplicated_iter,
            right_first: std::ptr::null_mut(),
            previous_item: self.first,
        }
    }
}

impl<I> Iterator for DedupSeq<I>
where
    I: Iterator,
    I::Item: Clone + PartialEq,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        let next_value = self.iter.next();
        if next_value.is_none() {
            if !self.right_first.is_null() {
                unsafe { self.right_first.write(self.previous_item.clone()) }
            }
            None
        } else if self.previous_item == next_value {
            self.next()
        } else {
            self.previous_item = next_value.clone();
            next_value
        }
    }
}
