//! Utilities functions to ease life of end users.
use crate::prelude::*;
use std;
use std::sync::atomic::{AtomicBool, Ordering};

/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
pub fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

/// iterate on starting_value * 2**i
pub fn powers(starting_value: usize) -> impl Iterator<Item = usize> {
    (0..).scan(starting_value, |state, _| {
        *state *= 2;
        Some(*state)
    })
}

pub struct AbortingDivisible<'a, I> {
    pub real_content: I,
    pub abort: &'a AtomicBool,
}

impl<'a, I: Divisible> Divisible for AbortingDivisible<'a, I> {
    type Power = I::Power;
    fn base_length(&self) -> usize {
        if self.abort.load(Ordering::Relaxed) {
            0
        } else {
            self.real_content.base_length()
        }
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.real_content.divide();
        (
            AbortingDivisible {
                real_content: left,
                abort: self.abort,
            },
            AbortingDivisible {
                real_content: right,
                abort: self.abort,
            },
        )
    }
}

impl<'a, I: DivisibleIntoBlocks> DivisibleIntoBlocks for AbortingDivisible<'a, I> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.real_content.divide_at(index);
        (
            AbortingDivisible {
                real_content: left,
                abort: self.abort,
            },
            AbortingDivisible {
                real_content: right,
                abort: self.abort,
            },
        )
    }
}

impl<'a, I: DivisibleAtIndex> DivisibleAtIndex for AbortingDivisible<'a, I> {}

impl<'a, I: IntoIterator> IntoIterator for AbortingDivisible<'a, I> {
    type IntoIter = I::IntoIter;
    type Item = I::Item;
    fn into_iter(self) -> Self::IntoIter {
        self.real_content.into_iter()
    }
}
