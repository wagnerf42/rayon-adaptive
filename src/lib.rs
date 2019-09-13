mod base;
mod dislocated;
mod iter;
pub mod prelude;
mod scheduler;
pub(crate) mod small_channel;
mod traits;
pub use base::repeat::repeat;
pub use base::successors::successors;

// TODO: by_blocks -> we need a method giving the blocks sizes
//
//
#[macro_use]
mod private;
pub(crate) use private_try::Try;

/// We hide the `Try` trait in a private module, as it's only meant to be a
/// stable clone of the standard library's `Try` trait, as yet unstable.
/// this snippet is taken directly from rayon.
mod private_try {
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
