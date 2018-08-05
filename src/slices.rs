//! We provide here `EdibleSlice` and `EatingIterator` for better composability.

use std::slice::IterMut;
use Divisible;

/// A slice you can consume slowly.
pub struct EdibleSlice<'a, T: 'a> {
    // the real underlying slice
    slice: &'a [T],
    // how much we used up to now
    used: usize,
}

impl<'a, T: 'a> EdibleSlice<'a, T> {
    /// Create a new `EdibleSlice` out of given slice.
    pub fn new(slice: &'a [T]) -> Self {
        EdibleSlice { slice, used: 0 }
    }
    /// Return what's left of the inner slice.
    fn remaining_slice(&self) -> &'a [T] {
        &self.slice[self.used..]
    }
    /// Return an iterator on remaining elements.
    /// When the iterator drops we remember what's left unused.
    ///
    /// # Examples
    ///
    /// ```
    /// use rayon_adaptive::EdibleSlice;
    /// let v = vec![0, 1, 2, 3, 4];
    /// // it needs to be mutable because inner position gets updated
    /// let mut slice = EdibleSlice::new(&v);
    /// let v1: Vec<u32> = slice.iter().take(3).cloned().collect();
    /// // second iterator picks up where last one stopped
    /// let v2: Vec<u32> = slice.iter().cloned().collect();
    /// assert_eq!(v1, vec![0, 1, 2]);
    /// assert_eq!(v2, vec![3, 4]);
    /// ```
    pub fn iter<'b>(&'b mut self) -> EatingIterator<'a, 'b, T> {
        EatingIterator {
            edible: self,
            eaten: 0,
        }
    }
}

impl<'a, T: 'a + Sync> Divisible for EdibleSlice<'a, T> {
    fn len(&self) -> usize {
        self.slice.len() - self.used
    }
    fn split(self) -> (Self, Self) {
        let splitting_index = self.used + self.remaining_slice().len() / 2;
        let (left_slice, right_slice) = self.slice.split_at(splitting_index);
        (
            EdibleSlice {
                slice: left_slice,
                used: self.used,
            },
            EdibleSlice {
                slice: right_slice,
                used: 0,
            },
        )
    }
}

/// Iterator on `EdibleSlice`.
/// Updates slice's state on drop.
pub struct EatingIterator<'a: 'b, 'b, T: 'a> {
    edible: &'b mut EdibleSlice<'a, T>,
    eaten: usize,
}

impl<'a: 'b, 'b, T: 'a> Iterator for EatingIterator<'a, 'b, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.edible.remaining_slice().len() == self.eaten {
            None
        } else {
            self.eaten += 1;
            Some(&self.edible.remaining_slice()[self.eaten - 1])
        }
    }
}

impl<'a: 'b, 'b, T: 'a> Drop for EatingIterator<'a, 'b, T> {
    fn drop(&mut self) {
        self.edible.used += self.eaten
    }
}

/// A mutable slice you can consume slowly.
pub struct EdibleSliceMut<'a, T: 'a> {
    // the real underlying slice
    slice: &'a mut [T],
    // how much we used up to now
    used: usize,
}

impl<'a, T: 'a> EdibleSliceMut<'a, T> {
    /// Create a new `EdibleSliceMut` out of given mutable slice.
    pub fn new(slice: &'a mut [T]) -> Self {
        EdibleSliceMut { slice, used: 0 }
    }
    /// Return what's left of the inner slice.
    fn remaining_slice<'b>(&'b mut self) -> &'b mut [T] {
        &mut self.slice[self.used..]
    }
    /// Return an iterator on remaining elements (mutable).
    /// When the iterator drops we remember what's left unused.
    pub fn iter<'b>(&'b mut self) -> EatingIteratorMut<'a, 'b, T> {
        EatingIteratorMut {
            edible: self,
            eaten: 0,
            iterator: self.remaining_slice().iter_mut(),
        }
    }
}

//TODO: factorize with other iterator using some more traits.
/// Iterator on `EdibleSlice`.
/// Updates slice's state on drop.
pub struct EatingIteratorMut<'a: 'b, 'b, T: 'a> {
    edible: &'b mut EdibleSliceMut<'a, T>,
    iterator: IterMut<'b, T>,
    eaten: usize,
}

impl<'a: 'b, 'b, T: 'a> Iterator for EatingIteratorMut<'a, 'b, T> {
    type Item = &'b mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let next_one = self.iterator.next();
        if next_one.is_some() {
            self.eaten += 1;
        }
        next_one
    }
}

impl<'a: 'b, 'b, T: 'a> Drop for EatingIteratorMut<'a, 'b, T> {
    fn drop(&mut self) {
        self.edible.used += self.eaten
    }
}
