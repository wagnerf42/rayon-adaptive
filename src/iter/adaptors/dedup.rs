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
    iter: Chain<I, std::option::IntoIter<I::Item>>,
    right_first: *mut Option<I::Item>,
    remaining_iterations: Option<usize>,
    last_yielded: Option<I::Item>,
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
        let (remaining_left, last_left) = left.divide_at(index - 1);
        // let (last_left, remaining_right) = right.divide_at(1);
        let last = last_left.to_sequential().next();
        let first_right = last.clone();

        (
            Dedup {
                first: self.first,
                iter: remaining_left,
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
        let iter_final = self.iter.extract_iter(size).chain(None);

        DedupSeq {
            iter: iter_final,
            right_first: &mut self.first as *mut Option<I::Item>,
            remaining_iterations: Some(size),
            last_yielded: self.first,
        }
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        let iterations = self.base_length();
        let iter_final = self.iter.to_sequential().chain(self.last);
        DedupSeq {
            iter: iter_final,
            right_first: std::ptr::null_mut(),
            remaining_iterations: iterations,
            last_yielded: self.first,
        }
    }
}

impl<I> Iterator for DedupSeq<I>
where
    I: Iterator,
    I::Item: Clone + PartialEq + Copy,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_iterations.unwrap_or(1) == 0 {
            // if this is the last iteration we need to propagate the value
            // to the other side (if any).
            if !self.right_first.is_null() {
                unsafe { self.right_first.write(self.last_yielded.clone()) }
            }
            None
        } else {
            self.remaining_iterations = self.remaining_iterations.map(|i| i - 1);
            let returned_value = self.iter.next();
            if self.last_yielded == returned_value {
                self.next()
            } else {
                self.last_yielded = returned_value;
                returned_value
            }
        }
    }
}
