use crate::Policy;
use std::mem;

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

/// Iterator on some `Divisible` input by blocks.
pub struct BlocksIterator<I: DivisibleIntoBlocks> {
    /// sizes of all the remaining blocks
    sizes: Box<Iterator<Item = usize>>,
    /// remaining input
    remaining: Option<I>,
}

impl<I: DivisibleIntoBlocks> Iterator for BlocksIterator<I> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_none() {
            // no input left
            return None;
        }

        let remaining_length = self.remaining.as_ref().base_length();
        if let Some(length) = remaining_length {
            if length == 0 {
                // no input left
                return None;
            }
        }

        let current_size = self.sizes.next();
        if let Some(size) = current_size {
            let remaining_input = self.remaining.take().unwrap();
            let (left, right) = remaining_input.divide_at(size);
            mem::replace(&mut self.remaining, Some(right));
            Some(left)
        } else {
            // no sizes left, return everything thats left to process
            self.remaining.take()
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
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
    /// Iterate on all blocks.
    fn blocks(self) -> BlocksIterator<Self> {
        let sizes = Box::new(self.base_length().into_iter());
        BlocksIterator {
            remaining: Some(self),
            sizes,
        }
    }
}

/// Implement for zippable stuff.
pub trait DivisibleAtIndex: DivisibleIntoBlocks {}
