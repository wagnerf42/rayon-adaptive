//! we implement parallel collects here.
use crate::divisibility::{IndexedPower, Power};
use crate::prelude::*;

/// Types which can be collected into from an indexed parallel iterator should implement this.
pub trait FromIndexedParallelIterator<T>
where
    T: Send,
{
    /// This defines a specialised collect method which should typically be faster than the blocked
    /// collect for unindexed parallel iterators
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
        I::Iter: ParallelIterator<Power = IndexedPower>;
}

impl<T: Send + Sync> FromIndexedParallelIterator<T> for Vec<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
        I::Iter: ParallelIterator<Power = IndexedPower>,
    {
        let mut final_vector = Vec::new();
        final_vector
            .as_mut_slice()
            .into_par_iter()
            .zip(par_iter)
            .for_each(|(dst, src)| {
                *dst = src;
            });
        final_vector
    }
}
