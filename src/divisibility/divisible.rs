use crate::Policy;
use std::iter::empty;
use std::marker::PhantomData;
use std::mem;

/// This is a marker type for specialization
pub struct BasicPower();
/// This is a marker type for specialization
pub struct BlockedPower();
/// This is a marker type for specialization
pub struct IndexedPower();

/// To constrain types a little bit all markers need to implement this.
pub trait Power: Send {} // TODO: why on earth is the compiler requesting that ?
impl Power for BasicPower {}
impl Power for BlockedPower {}
impl Power for IndexedPower {}

/// This is the first level of the divisibility traits hierarchy.
/// All parallel objects must at least implement this trait.
/// Note that this abstraction is stronger than parallel iterators and
/// will allow parallel operations on non-iterator objects.
pub trait Divisible<P: Power>: Sized {
    /// Return our size. This corresponds to the number of operations to be issued.
    /// For example *i.filter(f)* should have as size the number of elements in i before
    /// filtering. At size 0 nothing is left to do.
    /// Return None if size is infinite.
    fn base_length(&self) -> Option<usize>;
    /// Cut the `Divisible` into two parts.
    fn divide(self) -> (Self, Self);
    /// Cut the `Divisible` into two parts, if possible at given index.
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.divide() // TODO: should divide be the default or divide at ?
    }
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
    /// Get a sequential iterator on all our macro blocks.
    fn blocks(self) -> BlocksIterator<P, Self> {
        let sizes = Box::new(empty());
        BlocksIterator {
            remaining: Some(self),
            sizes,
            phantom: PhantomData,
        }
    }
}

/// Iterator on some `Divisible` input by blocks.
pub struct BlocksIterator<P: Power, I: Divisible<P>> {
    /// sizes of all the remaining blocks
    sizes: Box<Iterator<Item = usize>>,
    /// remaining input
    remaining: Option<I>,
    phantom: PhantomData<P>,
}

impl<I: Divisible<BasicPower>> Iterator for BlocksIterator<BasicPower, I> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        self.remaining.take()
    }
}

impl<I: Divisible<BlockedPower>> Iterator for BlocksIterator<BlockedPower, I> {
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
