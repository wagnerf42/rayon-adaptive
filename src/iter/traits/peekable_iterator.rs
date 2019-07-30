use crate::iter::ParallelMerge;
use crate::prelude::*;

/// A peekable iterator, allowing to peek at a specified index in the underlying data
pub trait PeekableIterator: IndexedParallelIterator {
    /// Peeks into the iterator without consuming it, returning the item at the specified location
    fn peek(&self, index: usize) -> Self::Item;
    /// Return next element.
    /// pre-condition: don't call if empty.
    fn next(&mut self) -> Option<Self::Item>;
}

/// Ordered Peekable iterators can be merged together.
pub trait MergeableIterator: PeekableIterator {
    /// Merge two ordered parallel iterators into one ordered parallel iterator.
    fn merge<J: PeekableIterator<Item = Self::Item>>(self, other: J) -> ParallelMerge<Self, J> {
        ParallelMerge {
            left: self,
            right: other,
        }
    }
}

impl<T: Ord + Send, I: PeekableIterator<Item = T>> MergeableIterator for I {}
