//! Parallel iterator on mutable slices.
use crate::prelude::*;

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
    type Item = &'a mut T;
}

impl<'extraction, 'a, T: 'a + Send> FinitePart<'extraction> for IterMut<'a, T> {
    type ParIter = IterMut<'a, T>;
    type SeqIter = std::slice::IterMut<'a, T>;
}

impl<'a, T: 'a + Send> ParallelIterator for IterMut<'a, T> {
    fn borrow_on_left_for<'extraction>(&'extraction mut self, size: usize) -> IterMut<'a, T> {
        let (left, right) = self.slice.take().unwrap().split_at_mut(size);
        self.slice = Some(right);
        IterMut { slice: Some(left) }
    }

    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
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
