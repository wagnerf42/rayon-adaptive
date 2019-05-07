//! Adaptive iterators

mod traits;
pub use traits::Edible;
pub use traits::{
    BasicParallelIterator, BlockedParallelIterator, IndexedParallelIterator, ParallelIterator,
};
mod range;

mod iterator_fold;
pub use iterator_fold::IteratorFold;
