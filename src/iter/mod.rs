//! Adaptive iterators

mod traits;
pub use traits::BaseIterator;
pub use traits::Edible;
mod parallel_iterator;
pub use parallel_iterator::ParallelIterator;
mod blocked_iterator;
pub use blocked_iterator::BlockedIterator;
