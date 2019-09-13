//! Parallel iterator on mutable slices.
use crate::prelude::*;

pub struct Iter<'a, T: 'a> {
    pub(crate) slice: &'a [T],
}

impl<'a, T: 'a> Divisible for Iter<'a, T> {
    fn is_divisible(&self) -> bool {
        self.slice.len() > 1
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.slice.len() / 2;
        let (left, right) = self.slice.split_at(mid);
        (Iter { slice: left }, Iter { slice: right })
    }
}

impl<'a, T: 'a + Sync + Send> ItemProducer for Iter<'a, T> {
    type Owner = Self;
    type Item = &'a T;
    type Power = Indexed;
}

impl<'e, 'a, T: 'a + Sync + Send> Borrowed<'e> for Iter<'a, T> {
    type ParIter = Iter<'a, T>;
    type SeqIter = std::slice::Iter<'a, T>;
}

impl<'a, T: 'a + Sync + Send> ParallelIterator for Iter<'a, T> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> Iter<'a, T> {
        let (left, right) = self.slice.split_at(size);
        self.slice = right;
        Iter { slice: left }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self as Borrowed<'e>>::SeqIter {
        let (left, right) = self.slice.split_at(size);
        self.slice = right;
        left.iter()
    }
}

impl<'a, T: 'a + Sync + Send> FiniteParallelIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.slice.len()
    }
}

impl<'a, T: 'a + Sync + Send> IntoParallelIterator for &'a [T] {
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

impl<'a, T: 'a> Divisible for IterMut<'a, T> {
    fn is_divisible(&self) -> bool {
        self.slice.as_ref().unwrap().len() > 1
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.slice.as_ref().unwrap().len() / 2;
        let (left, right) = self.slice.unwrap().split_at_mut(mid);
        (
            IterMut { slice: Some(left) },
            IterMut { slice: Some(right) },
        )
    }
}

impl<'a, T: 'a + Send> ItemProducer for IterMut<'a, T> {
    type Owner = Self;
    type Item = &'a mut T;
    type Power = Indexed;
}

impl<'e, 'a, T: 'a + Send> Borrowed<'e> for IterMut<'a, T> {
    type ParIter = IterMut<'a, T>;
    type SeqIter = std::slice::IterMut<'a, T>;
}

impl<'a, T: 'a + Send> ParallelIterator for IterMut<'a, T> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> IterMut<'a, T> {
        let (left, right) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(right);
        IterMut { slice: Some(left) }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self as Borrowed<'e>>::SeqIter {
        let (left, right) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(right);
        left.iter_mut()
    }
}

impl<'a, T: 'a + Send> FiniteParallelIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.slice.as_ref().unwrap().len()
    }
}

impl<'a, T: 'a + Send> IntoParallelIterator for &'a mut [T] {
    type Iter = IterMut<'a, T>;
    type Item = &'a mut T;
    fn into_par_iter(self) -> Self::Iter {
        IterMut { slice: Some(self) }
    }
}
