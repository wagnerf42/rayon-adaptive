use crate::prelude::*;

pub struct ParallelSlice<'a, T: 'a + Sync> {
    slice: &'a [T],
}

impl<'a, T: 'a + Sync> IntoIterator for ParallelSlice<'a, T> {
    type Item = &'a [T];
    type IntoIter = IntoIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter { par_slice: self }
    }
}

impl<'a, T: 'a + Sync> DivisibleParallelIterator for ParallelSlice<'a, T> {
    fn base_length(&self) -> usize {
        self.slice.len()
    }
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self {
        let (left, right) = self.slice.split_at(index);
        self.slice = right;
        ParallelSlice { slice: left }
    }
}

pub struct IntoIter<'a, T: Sync> {
    par_slice: ParallelSlice<'a, T>,
}

impl<'a, T: 'a + Sync> Iterator for IntoIter<'a, T> {
    type Item = &'a [T];
    fn next(&mut self) -> Option<Self::Item> {
        if self.par_slice.base_length() == 0 {
            None
        } else {
            let (left, right) = self.par_slice.slice.split_at(0);
            self.par_slice.slice = right;
            Some(left)
        }
    }
}
