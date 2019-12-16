//! Parallel iterator on mutable slices.
use crate::prelude::*;
use crate::traits::DivisibleIter;

/// Ordinary slices can also be turned into parallel iterators.
/// # Example:
/// ```
/// use rayon_adaptive::prelude::*;
/// let some_vec: Vec<u32> = (0..1000).collect();
/// assert_eq!((&some_vec[0..500]).into_par_iter().filter(|&e| e%2==0).sum::<u32>(), 62250)
/// ```
impl<'a, T: 'a + Sync> DivisibleParallelIterator for &'a [T] {
    fn base_length(&self) -> usize {
        self.len()
    }
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self {
        let (left, right) = self.split_at(index);
        *self = right;
        left
    }
}

impl<'a, T: 'a + Sync> IntoParallelIterator for &'a [T] {
    type Iter = DivisibleIter<Self>;
    type Item = &'a T;
    fn into_par_iter(self) -> Self::Iter {
        DivisibleIter { base: self }
    }
}

impl<'a, T: 'a> std::ops::Index<usize> for DivisibleIter<&'a [T]> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.base[index]
    }
}

//// mutable slices
impl<'a, T: 'a + Sync + Send> DivisibleParallelIterator for &'a mut [T] {
    fn base_length(&self) -> usize {
        self.len()
    }
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self {
        let len = self.len();
        let ptr = self.as_mut_ptr();

        // this unsafe code is copy-pasted from split_at_mut
        let (left, right) = unsafe {
            assert!(index <= len);

            (
                std::slice::from_raw_parts_mut(ptr, index),
                std::slice::from_raw_parts_mut(ptr.add(index), len - index),
            )
        };

        *self = right;
        left
    }
}

impl<'a, T: 'a + Sync + Send> IntoParallelIterator for &'a mut [T] {
    type Iter = DivisibleIter<Self>;
    type Item = &'a mut T;
    fn into_par_iter(self) -> Self::Iter {
        DivisibleIter { base: self }
    }
}

impl<'a, T: 'a> std::ops::Index<usize> for DivisibleIter<&'a mut [T]> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.base[index]
    }
}
