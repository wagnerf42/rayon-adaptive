use crate::help_work::HelpWork;
use crate::iter::{Cut, Work};
use std::iter::empty;
use std::mem;

/// This is a marker type for specialization
pub struct BasicPower();
/// This is a marker type for specialization
pub struct BlockedPower();
/// This is a marker type for specialization
pub struct IndexedPower();

/// To constrain types a little bit all markers need to implement this.
pub trait Power: Send {
    /// Power we have if we lose indexing.
    type NotIndexed: Power;
} // TODO: why on earth is the compiler requesting send ?
impl Power for BasicPower {
    type NotIndexed = BasicPower;
}
impl Power for BlockedPower {
    type NotIndexed = BlockedPower;
}
impl Power for IndexedPower {
    type NotIndexed = BlockedPower;
}
pub trait BlockedPowerOrMore: Power {}
impl BlockedPowerOrMore for BlockedPower {}
impl BlockedPowerOrMore for IndexedPower {}

/// This is the first level of the divisibility traits hierarchy.
/// All parallel objects must at least implement this trait.
/// Note that this abstraction is stronger than parallel iterators and
/// will allow parallel operations on non-iterator objects.
pub trait Divisible: Sized {
    /// What we can really do.
    type Power: Power;
    /// Return our size. This corresponds to the number of operations to be issued.
    /// For example *i.filter(f)* should have as size the number of elements in i before
    /// filtering. At size 0 nothing is left to do.
    /// Return None if size is infinite.
    fn base_length(&self) -> Option<usize>;
    /// Cut the `Divisible` into two parts.
    fn divide(self) -> (Self, Self) {
        let mid = self
            .base_length()
            .expect("cannot divide by default with no size")
            / 2;
        self.divide_at(mid)
    }
    /// Cut the `Divisible` into two parts, if possible at given index.
    fn divide_at(self, index: usize) -> (Self, Self);
    /// Return a sequential iterator on blocks of Self of given sizes.
    fn blocks<S: Iterator<Item = usize>>(self, sizes: S) -> BlocksIterator<Self, S> {
        BlocksIterator {
            sizes,
            remaining: Some(self),
        }
    }
    /// Work on ourselves piece by piece until length reaches 0.
    fn work<W: Fn(Self, usize) -> Self + Send + Clone>(self, work_op: W) -> Work<Self, W> {
        Work {
            remaining_input: Some(self),
            work_op,
        }
    }
    /// Get a parallel iterator on parts of self.
    fn cut(self) -> Cut<Self> {
        Cut { input: self }
    }
    /// Let's work, helping the sequential thread.
    fn with_help_work<H>(self, help_op: H) -> HelpWork<Self, H>
    where
        H: Fn(Self, usize) -> Self,
    {
        HelpWork {
            input: self,
            help_op,
            sizes: Box::new(empty()),
        }
    }
}

/// Iterator on some `Divisible` input by blocks.
pub struct BlocksIterator<I: Divisible, S: Iterator<Item = usize>> {
    /// sizes of all the remaining blocks
    pub(crate) sizes: S,
    /// remaining input
    pub(crate) remaining: Option<I>,
}

impl<I: Divisible, S: Iterator<Item = usize>> Iterator for BlocksIterator<I, S> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_none() {
            // no input left
            return None;
        }

        let remaining_length = self.remaining.as_ref().and_then(I::base_length);
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
