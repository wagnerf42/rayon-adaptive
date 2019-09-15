//! we implement parallel collects here.
use crate::divisibility::{Divisible, IndexedPower};
use crate::prelude::*;

/// Types which can be collected into from an indexed parallel iterator should implement this.
pub trait FromIndexedParallelIterator<T>: FromParallelIterator<T>
where
    T: Send,
{
    /// This defines a specialised collect method which should typically be faster than the blocked
    /// collect for unindexed parallel iterators
    fn from_indexed_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
        I::Iter: ParallelIterator<Power = IndexedPower>;
}

impl<T: Send + Sync> FromIndexedParallelIterator<T> for Vec<T> {
    /// collects into the vector
    fn from_indexed_par_iter<I>(par_iter: I) -> Self
    where
        I: IntoParallelIterator<Item = T>,
        I::Iter: ParallelIterator<Power = IndexedPower>,
    {
        let real_par_iter = par_iter.into_par_iter();
        let vec_len = real_par_iter
            .base_length()
            .expect("IndexedParallelIterator refused to give a base length");
        //println!("Length of iter is {}", vec_len);
        let mut final_vector = Vec::with_capacity(vec_len);
        unsafe {
            final_vector.set_len(vec_len);
        }
        let myslice = final_vector.as_mut_slice();
        real_par_iter
            .zip(myslice.into_par_iter())
            .for_each(|(src, dst)| {
                unsafe { std::ptr::write(dst, src) };
            });
        final_vector
    }
}
