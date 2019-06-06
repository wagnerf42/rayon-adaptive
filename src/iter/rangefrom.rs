//! Parallel iterators on unbounded ranges !
use crate::divisibility::IndexedPower;
use crate::prelude::*;
use std::ops::{Range, RangeFrom};

/// Parallel iterator on unbounded range.
pub enum RangeFromParIter<E> {
    Bounded(Range<E>),
    UnBounded(RangeFrom<E>),
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
    type SequentialIterator = Range<usize>;
    type Item = usize;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        match self {
            RangeFromParIter::Bounded(r) => {
                let end = r.start + size;
                let iter = r.start..end;
                r.start = end;
                iter
            }
            RangeFromParIter::UnBounded(r) => {
                let end = r.start + size;
                let iter = r.start..end;
                r.start = end;
                iter
            }
        }
    }
    fn to_sequential(mut self) -> Self::SequentialIterator {
        let size = self
            .base_length()
            .expect("TODO: implement to_sequential in infinite ranges");
        self.extract_iter(size)
    }
}

impl IntoParallelIterator for RangeFrom<usize> {
    type Iter = RangeFromParIter<usize>;
    type Item = usize;
    fn into_par_iter(self) -> Self::Iter {
        RangeFromParIter::UnBounded(self)
    }
}
