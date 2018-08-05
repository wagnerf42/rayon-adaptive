//! We provide here `EdibleSlice` and `EatingIterator` for better composability.

use std::slice::Iter;
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
    pub fn iter<'b>(&'b mut self) -> EatingIterator<'b, T> {
        let used = self.used;
        EatingIterator {
            used: &mut self.used,
            iterator: self.slice[used..].iter(),
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
pub struct EatingIterator<'a, T: 'a> {
    used: &'a mut usize,
    iterator: Iter<'a, T>,
}

impl<'a, T: 'a> Iterator for EatingIterator<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let next_one = self.iterator.next();
        if next_one.is_some() {
            *self.used += 1;
        }
        next_one
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
    /// Take a look at next element.
    pub fn peek<'b>(&'b mut self) -> Option<&'b mut T> {
        self.slice.get_mut(self.used)
    }
    /// Get back the whole slice.
    pub fn slice(self) -> &'a mut [T] {
        self.slice
    }
    /// Return what's left of the inner slice.
    fn remaining_slice<'b>(&'b mut self) -> &'b mut [T] {
        &mut self.slice[self.used..]
    }
    /// Return an iterator on remaining elements (mutable).
    /// When the iterator drops we remember what's left unused.
    pub fn iter_mut<'b>(&'b mut self) -> EatingIteratorMut<'b, T> {
        let used = self.used;
        EatingIteratorMut {
            used: &mut self.used,
            iterator: self.slice[used..].iter_mut(),
        }
    }
}

impl<'a, T: 'a + Sync + Send> Divisible for EdibleSliceMut<'a, T> {
    fn len(&self) -> usize {
        self.slice.len() - self.used
    }
    fn split(mut self) -> (Self, Self) {
        let splitting_index = self.used + self.remaining_slice().len() / 2;
        let (left_slice, right_slice) = self.slice.split_at_mut(splitting_index);
        (
            EdibleSliceMut {
                slice: left_slice,
                used: self.used,
            },
            EdibleSliceMut {
                slice: right_slice,
                used: 0,
            },
        )
    }
}

//TODO: factorize with other iterator using some more traits.
/// Iterator on `EdibleSlice`.
/// Updates slice's state on drop.
pub struct EatingIteratorMut<'a, T: 'a> {
    used: &'a mut usize,
    iterator: IterMut<'a, T>,
}

impl<'a, T: 'a> Iterator for EatingIteratorMut<'a, T> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<Self::Item> {
        let next_one = self.iterator.next();
        if next_one.is_some() {
            *self.used += 1;
        }
        next_one
    }
}
