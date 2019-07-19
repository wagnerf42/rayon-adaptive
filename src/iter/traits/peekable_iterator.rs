use crate::prelude::*;

pub trait PeekableIterator: ParallelIterator {
    fn peek(&self, index: usize) -> Option<Self::Item>;
}
