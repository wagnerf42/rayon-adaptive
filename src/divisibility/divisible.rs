use crate::Policy;
use std::iter::empty;

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
        if let Some(l) = remaining_length {
            if l == 0 {
                return None;
            }
        }
        let current_size = self.sizes.next().unwrap_or(remaining_length.unwrap_or(0));
        let (current_block, remaining_blocks) = self.remaining.divide_at(current_size);
        self.remaining = remaining_blocks;
        Some(current_block)
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
        BlocksIterator {
            remaining: self,
            sizes: Box::new(empty()),
        }
    }
}
