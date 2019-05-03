//! Adaptive iterators

mod traits;
pub(crate) use traits::BaseIterator;
pub use traits::Edible;
mod blocked_iterator;
mod parallel_iterator;
