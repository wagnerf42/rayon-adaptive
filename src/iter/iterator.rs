//! `ParIter` structure. This is an empty shell to force the use of into_par_iter on basic types.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};

/// `ParIter` structure. This is an empty shell to force the use of into_par_iter on basic types.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P)]
#[item(I::Item)]
pub struct ParIter<P, I: Divisible<P>> {
    base: I,
}
