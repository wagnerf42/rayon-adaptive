use crate::iter::ParallelMerge;
use crate::prelude::*;
use std::marker::PhantomData;
use std::ops::Index;

/// Ordered Index iterators can be merged together.
pub trait MergeableIterator<T>: ParallelIterator<Power = Indexed>
where
    T: Ord,
    for<'e> <Self as ParBorrowed<'e>>::Iter: Index<usize, Output = T>,
{
    /// Merge two ordered parallel iterators into one ordered parallel iterator.
    fn merge<J>(self, other: J) -> ParallelMerge<T, Self, J>
    where
        J: ParallelIterator<Power = Indexed, Item = Self::Item>,
        for<'e> <J as ParBorrowed<'e>>::Iter: Index<usize, Output = T>,
    {
        ParallelMerge {
            left: self,
            right: other,
            phantom: PhantomData,
        }
    }
}

impl<I, T> MergeableIterator<T> for I
where
    T: Ord,
    I: ParallelIterator<Power = Indexed>,
    for<'e> <I as ParBorrowed<'e>>::Iter: Index<usize, Output = T>,
{
}
