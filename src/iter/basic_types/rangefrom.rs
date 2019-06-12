//! Parallel iterators on unbounded ranges !
use crate::divisibility::IndexedPower;
use crate::prelude::*;
#[cfg(nightly)]
use std::ops::Try;
use std::ops::{Range, RangeFrom};

/// Parallel iterator on unbounded range.
pub enum RangeFromParIter<E> {
    Bounded(Range<E>),
    UnBounded(RangeFrom<E>),
}

/// Sequential iterator on unbounded range.
pub enum RangeFromSeqIter<E> {
    Bounded(Range<E>),
    UnBounded(RangeFrom<E>),
}

impl Iterator for RangeFromSeqIter<usize> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            RangeFromSeqIter::Bounded(ref mut r) => {
                if r.start == r.end {
                    None
                } else {
                    let value = r.start;
                    r.start += 1;
                    Some(value)
                }
            }
            RangeFromSeqIter::UnBounded(ref mut r) => {
                let value = r.start;
                r.start += 1;
                Some(value)
            }
        }
    }
    #[cfg(nightly)]
    fn try_fold<B, F, R>(&mut self, init: B, f: F) -> R
    where
        F: FnMut(B, Self::Item) -> R,
        R: Try<Ok = B>,
    {
        match self {
            RangeFromSeqIter::Bounded(r) => r.try_fold(init, f),
            RangeFromSeqIter::UnBounded(r) => r.try_fold(init, f),
        }
    }
}

impl Divisible for RangeFromParIter<usize> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        match self {
            RangeFromParIter::Bounded(r) => Some(r.len()),
            RangeFromParIter::UnBounded(_) => None,
        }
    }
    fn divide(self) -> (Self, Self) {
        match self {
            RangeFromParIter::Bounded(r) => {
                let mid = (r.start + r.end) / 2;
                (
                    RangeFromParIter::Bounded(r.start..mid),
                    RangeFromParIter::Bounded(mid..r.end),
                )
            }
            RangeFromParIter::UnBounded(_) => panic!("divided infinite range iterator directly"),
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        match self {
            RangeFromParIter::Bounded(r) => (
                RangeFromParIter::Bounded(r.start..r.start + index),
                RangeFromParIter::Bounded(r.start + index..r.end),
            ),
            RangeFromParIter::UnBounded(r) => (
                RangeFromParIter::Bounded(r.start..r.start + index),
                RangeFromParIter::UnBounded(r.start + index..),
            ),
        }
    }
}

impl ParallelIterator for RangeFromParIter<usize> {
    type SequentialIterator = RangeFromSeqIter<usize>;
    type Item = usize;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        match self {
            RangeFromParIter::Bounded(r) => {
                let end = r.start + size;
                let iter = r.start..end;
                r.start = end;
                RangeFromSeqIter::Bounded(iter)
            }
            RangeFromParIter::UnBounded(r) => {
                let end = r.start + size;
                let iter = r.start..end;
                r.start = end;
                RangeFromSeqIter::Bounded(iter)
            }
        }
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        match self {
            RangeFromParIter::Bounded(r) => RangeFromSeqIter::Bounded(r),
            RangeFromParIter::UnBounded(r) => RangeFromSeqIter::UnBounded(r),
        }
    }
}

impl IntoParallelIterator for RangeFrom<usize> {
    type Iter = RangeFromParIter<usize>;
    type Item = usize;
    fn into_par_iter(self) -> Self::Iter {
        RangeFromParIter::UnBounded(self)
    }
}
