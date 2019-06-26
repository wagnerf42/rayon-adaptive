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
    ByBlocks, Cap, Chain, Dedup, DepthFirst, Filter, FilterMap, FineLog, FlatMap, FlatMapSeq, Fold,
    IteratorFold, Levels, Log, Map, Partition, Take, TryFold, WithPolicy, Zip,
};

// functions
mod functions;
pub use functions::successors;

pub(crate) use private::Try;

/// We hide the `Try` trait in a private module, as it's only meant to be a
/// stable clone of the standard library's `Try` trait, as yet unstable.
/// this snippet is taken directly from rayon.
mod private {
    /// Clone of `std::ops::Try`.
    ///
    /// Implementing this trait is not permitted outside of `rayon`.
    pub trait Try {
        private_decl! {}

        type Ok;
        type Error;
        fn into_result(self) -> Result<Self::Ok, Self::Error>;
        fn from_ok(v: Self::Ok) -> Self;
        fn from_error(v: Self::Error) -> Self;
    }

    impl<T> Try for Option<T> {
        private_impl! {}

        type Ok = T;
        type Error = ();

        fn into_result(self) -> Result<T, ()> {
            self.ok_or(())
        }
        fn from_ok(v: T) -> Self {
            Some(v)
        }
        fn from_error(_: ()) -> Self {
            None
        }
    }

    impl<T, E> Try for Result<T, E> {
        private_impl! {}

        type Ok = T;
        type Error = E;

        fn into_result(self) -> Result<T, E> {
            self
        }
        fn from_ok(v: T) -> Self {
            Ok(v)
        }
        fn from_error(v: E) -> Self {
            Err(v)
        }
    }
}
