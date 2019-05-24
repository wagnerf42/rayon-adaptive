//! we implement parallel collects here.
use crate::prelude::*;
use std::collections::LinkedList;
use std::iter::once;

/// Types which can be collected into from a parallel iterator should implement this.
pub trait FromParallelIterator<T>
where
    T: Send,
{
    /// Turn a parallel iterator into a collection.
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>;
}

impl<T: Send> FromParallelIterator<T> for Vec<T> {
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        let par_iter = par_iter.into_par_iter();
        // for now just a dumb version
        let mut blocks = par_iter
            .fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .map(|v| once(v).collect::<LinkedList<Vec<T>>>())
            .reduce(LinkedList::new, |mut l1, mut l2| {
                l1.append(&mut l2);
                l1
            })
            .into_iter();
        let mut final_vector = blocks.next().unwrap();
        for block in blocks {
            final_vector.extend(block)
        }
        final_vector
    }
}
