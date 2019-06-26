//! Implementing a iterator's tasks logger with rayon-logs.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::Divisible;
use std::ops::Drop;

/// Logging iterator (at fine detail) obtained from the `fine_log` method on `ParallelIterator`.
#[derive(Divisible)]
#[power(I::Power)]
#[trait_bounds(I: ParallelIterator)]
pub struct FineLog<I> {
    pub(crate) iterator: I,
    #[divide_by(clone)]
    pub(crate) tag: &'static str,
}

/// Sequential Logged Iterator.
pub struct LoggedIterator<I> {
    iterator: I,
    tag: &'static str,
    size: usize,
}

impl<I: Iterator> Iterator for LoggedIterator<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

impl<I> Drop for LoggedIterator<I> {
    fn drop(&mut self) {
        #[cfg(feature = "logs")]
        rayon_logs::end_subgraph(self.tag, self.size)
    }
}

impl<I: ParallelIterator> ParallelIterator for FineLog<I> {
    type SequentialIterator = LoggedIterator<I::SequentialIterator>;
    type Item = I::Item;
    fn to_sequential(self) -> Self::SequentialIterator {
        #[cfg(feature = "logs")]
        rayon_logs::start_subgraph(self.tag);
        let remaining_length = self.iterator.base_length().unwrap_or(1); // TODO: is it ok to default to 1 ?
        LoggedIterator {
            iterator: self.iterator.to_sequential(),
            tag: self.tag,
            size: remaining_length,
        }
    }
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        #[cfg(feature = "logs")]
        rayon_logs::start_subgraph(self.tag);
        LoggedIterator {
            iterator: self.iterator.extract_iter(size),
            tag: self.tag,
            size,
        }
    }
    fn policy(&self) -> Policy {
        self.iterator.policy()
    }
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        self.iterator.blocks_sizes()
    }
}
