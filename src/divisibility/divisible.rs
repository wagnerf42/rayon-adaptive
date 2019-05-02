use crate::Policy;
use std::iter::empty;
use std::ptr;

/// This is the first level of the divisibility traits hierarchy.
/// All parallel objects must at least implement this trait.
/// Note that this abstraction is stronger than parallel iterators and
/// will allow parallel operations on non-iterator objects.
pub trait Divisible: Sized {
    /// Return our size. This corresponds to the number of operations to be issued.
    /// For example *i.filter(f)* should have as size the number of elements in i before
    /// filtering. At size 0 nothing is left to do.
    fn base_length(&self) -> usize;
    /// Cut the `Divisible` into two parts.
    fn divide(self) -> (Self, Self);
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
}

/// Iterator on some `Divisible` input by blocks by blocks.
pub struct BlocksIterator<I: DivisibleIntoBlocks> {
    /// sizes of all the remaining blocks
    sizes: Box<Iterator<Item = usize>>,
    /// remaining input
    remaining: I,
}

impl<I: DivisibleIntoBlocks> Iterator for BlocksIterator<I> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        let remaining_length = self.remaining.base_length();
        if let Some(length) = remaining_length {
            if length == 0 {
                return None;
            }
        }
        let current_size = self.sizes.next();
        if let Some(size) = current_size {
            let current_block = self.remaining.cut_left_at(size);
            Some(current_block)
        } else {
            None
        }
    }
}

/// This is the second level of the divisibility traits hierarchy.
/// Most objects can be divided at specified indices.
/// We also allow infinite sizes like infinite ranges or repeat_with.
pub trait DivisibleIntoBlocks: Sized {
    /// Return our size. This corresponds to the number of operations to be issued.
    /// For example *i.filter(f)* should have as size the number of elements in i before
    /// filtering. At size 0 nothing is left to do.
    /// Infinite sizes should return *None*.
    fn base_length(&self) -> Option<usize>;
    /// Cut the `Divisible` into two parts at specified index.
    fn divide_at(self, index: usize) -> (Self, Self);
    /// Divide ourselves keeping right part in self.
    /// Returns the left part.
    /// NB: this is useful for iterators creation.
    fn cut_left_at(&mut self, index: usize) -> Self {
        // there is a lot of unsafe going on here.
        // I think it's ok. rust uses the same trick for moving iterators (vecs for example)
        unsafe {
            let my_copy = ptr::read(self);
            let (left, right) = my_copy.divide_at(index);
            let pointer_to_self = self as *mut Self;
            ptr::write(pointer_to_self, right);
            left
        }
    }

    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
    /// Iterate on all blocks.
    fn blocks(self) -> BlocksIterator<Self> {
        BlocksIterator {
            remaining: self,
            sizes: Box::new(empty()),
        }
    }
}
