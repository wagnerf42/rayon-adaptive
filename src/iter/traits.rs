//! Iterator governing traits.
//! `Edible` allows for a step by step extraction of sequential work from parallel iterator.
//! `BaseIterator` allows for code specialisation.
use crate::prelude::*;

/// We can produce sequential iterators to be eaten slowly.
pub trait Edible: Sized {
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator);
}

/// This traits enables to implement all basic methods for all type of iterators.
pub(crate) trait BaseIterator: Divisible + Edible {
    type BlocksIterator: Iterator<Item = Self>;
    fn blocks(self) -> Self::BlocksIterator;
    fn max(self) -> <<Self as Edible>::SequentialIterator as Iterator>::Item {
        unimplemented!()
    }
}
