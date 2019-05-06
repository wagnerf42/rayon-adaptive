//! Adaptive iterators

mod traits;
pub use traits::Edible;
pub use traits::{
    BasicParallelIterator, BlockedParallelIterator, IndexedParallelIterator, ParallelIterator,
};
mod range;
