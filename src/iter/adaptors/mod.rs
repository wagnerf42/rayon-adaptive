//! Adaptor types for parallel iterators.
// adaptors
mod iterator_fold;
pub use iterator_fold::IteratorFold;
mod with_policy;
pub use with_policy::WithPolicy;
mod by_blocks;
pub use by_blocks::ByBlocks;
mod fold;
pub use fold::Fold;
mod map;
pub use map::Map;
mod flat_map_seq;
pub use flat_map_seq::FlatMapSeq;
mod flat_map;
pub use flat_map::FlatMap;
mod filter_map;
pub use filter_map::FilterMap;
mod zip;
pub use zip::Zip;
mod interruptible;
pub use interruptible::Interruptible;
mod cap;
pub use cap::Cap;
