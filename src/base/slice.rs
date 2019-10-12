//! Parallel iterator on mutable slices.
use crate::prelude::*;

pub struct Iter<'a, T: 'a + Sync> {
    slice: &'a [T],
}

impl<'a, T: 'a + Sync> IntoIterator for Iter<'a, T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.slice.into_iter()
    }
}

impl<'a, T: 'a + Sync> DivisibleParallelIterator for Iter<'a, T> {
    fn base_length(&self) -> usize {
        self.slice.len()
    }
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self {
        let (left, right) = self.slice.split_at(index);
        self.slice = right;
        Iter { slice: left }
    }
}

/// Ordinary slices can also be turned into parallel iterators.
/// # Example:
/// ```
/// let some_vec: Vec<u32> = (0..1000).collect();
/// assert_eq!((&some_vec[0..500]).into_par_iter().filter(|&e| e%2==0).sum::<u32>(), 62250)
/// ```
impl<'a, T: 'a + Sync> IntoParallelIterator for &'a [T] {
    type Iter = Iter<'a, T>;
    type Item = &'a T;
    fn into_par_iter(self) -> Self::Iter {
        Iter { slice: self }
    }
}
//
//// mutable slices
//
pub struct IterMut<'a, T: 'a> {
    pub(crate) slice: Option<&'a mut [T]>, // TODO: this option is only here to avoid an unsafe
}

impl<'a, T: 'a + Send> ItemProducer for IterMut<'a, T> {
    type Item = &'a mut T;
}

impl<'a, T: 'a + Send> Powered for IterMut<'a, T> {
    type Power = Indexed;
}

impl<'e, 'a, T: 'a + Send> ParBorrowed<'e> for IterMut<'a, T> {
    type Iter = IterMut<'a, T>;
}

impl<'e, 'a, T: 'a + Send> SeqBorrowed<'e> for IterMut<'a, T> {
    type Iter = std::slice::IterMut<'a, T>;
}

impl<'a, T: 'a + Send> Divisible for IterMut<'a, T> {
    fn should_be_divided(&self) -> bool {
        self.slice.as_ref().unwrap().len() > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mid = self.slice.as_ref().unwrap().len() / 2;
        let (left, right) = self.slice.take().unwrap().split_at_mut(mid);
        (
            IterMut { slice: Some(left) },
            IterMut { slice: Some(right) },
        )
    }
}

impl<'a, T: 'a + Send> BorrowingParallelIterator for IterMut<'a, T> {
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        let (left, right) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(right);
        left.iter_mut()
    }
    fn iterations_number(&self) -> usize {
        self.slice.as_ref().unwrap().len()
    }
}

impl<'a, T: 'a + Send> ParallelIterator for IterMut<'a, T> {
    fn bound_iterations_number(&self, size: usize) -> usize {
        std::cmp::min(self.slice.as_ref().unwrap().len(), size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> IterMut<'a, T> {
        let (left, right) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(right);
        IterMut { slice: Some(left) }
    }
}

impl<'a, T: 'a + Send> IntoParallelIterator for &'a mut [T] {
    type Iter = IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_par_iter(self) -> Self::Iter {
        IterMut { slice: Some(self) }
    }
}
