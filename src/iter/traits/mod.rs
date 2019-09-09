//! All traits related to parallel iterators:
//! - ParallelIterator
//! - IntoParallelIterator
//! - ...
pub(crate) mod from_indexed_parallel_iterator;
pub(crate) mod from_parallel_iterator;
pub(crate) mod into_parallel_iterator;
pub(crate) mod parallel_iterator;
pub(crate) mod peekable_iterator;
