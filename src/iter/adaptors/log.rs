//! Implementing an iterator's tasks logger with rayon-logs (not detailed version).
use crate::prelude::*;
use crate::Policy;
use std::ops::Drop;

#[cfg(feature = "logs")]
/// Logging iterator (at task detail) obtained from the `log` method on `ParallelIterator`.
pub struct Log<I> {
    pub(crate) iterator: I,
    pub(crate) tag: &'static str,
    pub(crate) already_used: usize,
}

#[cfg(not(feature = "logs"))]
/// Logging iterator (at task detail) obtained from the `log` method on `ParallelIterator`.
pub struct Log<I> {
    pub(crate) iterator: I,
}

impl<I: ParallelIterator> Divisible for Log<I> {
    type Power = I::Power;
    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        #[cfg(feature = "logs")]
        {
            if self.already_used != 0 {
                rayon_logs::end_subgraph(self.tag, self.already_used);
                self.already_used = 0;
            }
        }
        let (left, right) = self.iterator.divide_at(index);
        self.iterator = left;
        let r;
        #[cfg(feature = "logs")]
        {
            let tag = self.tag;
            r = (
                self,
                Log {
                    iterator: right,
                    tag,
                    already_used: 0,
                },
            );
        }
        #[cfg(not(feature = "logs"))]
        {
            r = (self, Log { iterator: right });
        }
        r
    }
}

#[cfg(feature = "logs")]
/// Sequential Logged Iterator.
pub struct LoggedIterator<I> {
    iterator: I,
    tag: &'static str,
    size: usize,
    log: bool, // only last iterator will be logged, we need to mark it
}

#[cfg(not(feature = "logs"))]
/// Sequential Logged Iterator.
pub struct LoggedIterator<I> {
    iterator: I,
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
        {
            if self.log {
                rayon_logs::end_subgraph(self.tag, self.size)
            }
        }
    }
}

impl<I: ParallelIterator> ParallelIterator for Log<I> {
    type SequentialIterator = LoggedIterator<I::SequentialIterator>;
    type Item = I::Item;
    fn to_sequential(self) -> Self::SequentialIterator {
        let r;
        #[cfg(feature = "logs")]
        {
            if self.already_used == 0 {
                rayon_logs::start_subgraph(self.tag);
            }
            let remaining_length = self.iterator.base_length().unwrap_or(1); // TODO: is it ok to default to 1 ?
            r = LoggedIterator {
                iterator: self.iterator.to_sequential(),
                tag: self.tag,
                size: self.already_used + remaining_length,
                log: true,
            }
        }
        #[cfg(not(feature = "logs"))]
        {
            r = LoggedIterator {
                iterator: self.iterator.to_sequential(),
            }
        }
        r
    }
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let r;
        #[cfg(feature = "logs")]
        {
            if self.already_used == 0 {
                rayon_logs::start_subgraph(self.tag);
            }
            self.already_used += size;
            r = LoggedIterator {
                iterator: self.iterator.extract_iter(size),
                tag: self.tag,
                size,
                log: false,
            }
        }
        #[cfg(not(feature = "logs"))]
        {
            r = LoggedIterator {
                iterator: self.iterator.extract_iter(size),
            }
        }
        r
    }
    fn policy(&self) -> Policy {
        self.iterator.policy()
    }
    fn blocks_sizes(&mut self) -> Box<dyn Iterator<Item = usize>> {
        self.iterator.blocks_sizes()
    }
}
