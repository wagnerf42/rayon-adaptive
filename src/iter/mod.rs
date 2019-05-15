//! Adaptive iterators

mod traits;
pub use traits::Edible;
pub use traits::{
    BasicParallelIterator, BlockedParallelIterator, IndexedParallelIterator, ParallelIterator,
};
// basic types are edible
mod range;
mod slice;

// adaptors
mod iterator_fold;
pub use iterator_fold::IteratorFold;
mod with_policy;
pub use with_policy::WithPolicy;
mod by_blocks;
pub use by_blocks::ByBlocks;
mod fold;
pub use fold::Fold;
mod work;
pub use work::Work;
mod map;
pub use map::Map;
