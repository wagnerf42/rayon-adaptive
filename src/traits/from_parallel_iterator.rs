use crate::prelude::*;
use std::collections::LinkedList;

pub trait FromParallelIterator<T>
where
    T: Send,
{
    /// Creates an instance of the collection from the parallel iterator `par_iter`.
    ///
    /// If your collection is not naturally parallel, the easiest (and
    /// fastest) way to do this is often to collect `par_iter` into a
    /// [`LinkedList`] or other intermediate data structure and then
    /// sequentially extend your collection. However, a more 'native'
    /// technique is to use the [`par_iter.fold`] or
    /// [`par_iter.fold_with`] methods to create the collection.
    /// Alternatively, if your collection is 'natively' parallel, you
    /// can use `par_iter.for_each` to process each element in turn.
    ///
    /// [`LinkedList`]: https://doc.rust-lang.org/std/collections/struct.LinkedList.html
    /// [`par_iter.fold`]: trait.ParallelIterator.html#method.fold
    /// [`par_iter.fold_with`]: trait.ParallelIterator.html#method.fold_with
    /// [`par_iter.for_each`]: trait.ParallelIterator.html#method.for_each
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>;
}

impl<T> FromParallelIterator<T> for Vec<T>
where
    T: Send,
{
    fn from_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
    {
        let i = par_iter.into_par_iter();
        let vectors_list = i
            .fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .map(|v| std::iter::once(v).collect::<LinkedList<_>>())
            .reduce(LinkedList::new, |mut l1, mut l2| {
                l1.append(&mut l2);
                l1
            });
        let mut seq_iter = vectors_list.into_iter();
        let final_vec = seq_iter.next().unwrap_or_else(Vec::new);
        seq_iter.fold(final_vec, |mut final_v, mut v| {
            final_v.append(&mut v);
            final_v
        })
    }
}
