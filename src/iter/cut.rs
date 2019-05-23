//! Parallel iterator on pieces of a `Divisible`. This can be useful when divisions cost nothing.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator};
use std::iter::{once, Once};
use std::marker::PhantomData;

/// `ParallelIterator` on divided `Divisible`.
#[derive(Divisible, IntoIterator)]
#[power(P)]
#[item(D)]
#[trait_bounds(P: Power, D:Divisible<P> + Send)]
pub struct Cut<P, D> {
    pub(crate) input: D,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, D: Divisible<P> + Send> ParallelIterator<P> for Cut<P, D> {
    type Item = D;
    type SequentialIterator = Once<D>;
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        let (left, right) = self.input.divide_at(size);
        (
            once(left),
            Cut {
                input: right,
                phantom: PhantomData,
            },
        )
    }
}
