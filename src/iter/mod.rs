//! Adaptive iterators

mod traits;
pub use traits::from_parallel_iterator::FromParallelIterator;
pub use traits::into_parallel_iterator::IntoParallelIterator;
pub use traits::parallel_iterator::{
    BasicParallelIterator, BlockedOrMoreParallelIterator, BlockedParallelIterator,
    IndexedParallelIterator, ParallelIterator,
};

// basic types are defined here.
mod basic_types;

// special types
mod work;
pub use work::Work;
mod cut;
pub use cut::Cut;

mod adaptors;
pub use adaptors::{
    ByBlocks, Cap, Filter, FilterMap, FlatMap, FlatMapSeq, Fold, Interruptible, IteratorFold, Map,
    WithPolicy, Zip,
};

// functions
mod functions;
pub use functions::successors;
