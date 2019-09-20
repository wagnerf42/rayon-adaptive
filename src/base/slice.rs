//! Parallel iterator on mutable slices.
use crate::prelude::*;

pub struct Iter<'a, T: 'a> {
    slice: &'a [T],
}

impl<'a, T: 'a + Sync> ItemProducer for Iter<'a, T> {
    type Item = &'a T;
}

impl<'e, 'a, T: 'a + Sync> ParBorrowed<'e> for Iter<'a, T> {
    type Iter = Iter<'a, T>;
}

impl<'e, 'a, T: 'a + Sync> SeqBorrowed<'e> for Iter<'a, T> {
    type Iter = std::slice::Iter<'a, T>;
}

impl<'a, T: 'a + Sync> Divisible for Iter<'a, T> {
    fn should_be_divided(&self) -> bool {
        self.slice.len() > 1
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.slice.len() / 2;
        let (left, right) = self.slice.split_at(mid);
        (Iter { slice: left }, Iter { slice: right })
    }
}

impl<'a, T: 'a + Sync> BorrowingParallelIterator for Iter<'a, T> {
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        let (left, right) = self.slice.split_at(size);
        self.slice = right;
        left.into_iter()
    }
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, T: 'a + Sync> ParallelIterator for Iter<'a, T> {
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        let (left, right) = self.slice.split_at(size);
        self.slice = right;
        Iter { slice: left }
    }
}

impl<'a, T: 'a + Sync> IndexedParallelIterator for Iter<'a, T> {}

impl<'a, T: 'a + Sync> IntoParallelIterator for &'a [T] {
    type Iter = Iter<'a, T>;
    type Item = &'a T;
    fn into_par_iter(self) -> Self::Iter {
        Iter { slice: self }
    }
}

// mutable slices

pub struct IterMut<'a, T: 'a> {
    pub(crate) slice: Option<&'a mut [T]>, // TODO: this option is only here to avoid an unsafe
}

impl<'a, T: 'a + Send> ItemProducer for IterMut<'a, T> {
    type Item = &'a mut T;
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
    fn len(&self) -> usize {
        self.slice.as_ref().unwrap().len()
    }
}

impl<'a, T: 'a + Send> ParallelIterator for IterMut<'a, T> {
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

impl<'a, T: 'a + Send> IndexedParallelIterator for IterMut<'a, T> {}
