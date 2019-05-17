//! Adaptive iterators

mod traits;
pub use traits::into_parallel_iterator::IntoParallelIterator;
pub use traits::parallel_iterator::{
    BasicParallelIterator, BlockedParallelIterator, IndexedParallelIterator, ParallelIterator,
};

// basic types are
mod range;
mod rangefrom;
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
mod flatmap;
pub use flatmap::FlatMap;
mod filter_map;
pub use filter_map::FilterMap;
