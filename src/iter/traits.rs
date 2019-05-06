//! Iterator governing traits.
//! `Edible` allows for a step by step extraction of sequential work from parallel iterator.
//! `BaseIterator` allows for code specialisation.
use crate::prelude::*;
use crate::schedulers::schedule;

/// We can produce sequential iterators to be eaten slowly.
pub trait Edible: Sized + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send;
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator<Item = Self::Item>;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator);
}

/// This traits enables to implement all basic methods for all type of iterators.
pub trait BaseIterator: Divisible + Edible {
    /// Type for blocks iterator (can be Once).
    type BlocksIterator: Iterator<Item = Self>;
    /// Iterate on all our blocks.
    fn blocks(self) -> Self::BlocksIterator;
    //fn iterator_map<F>(self, F: map_op) -> ParallelIterator<IteratorMap<Self>> {
    //    unimplemented!()
    //}
    /// Reduce with call to scheduler.
    fn reduce<OP, ID>(
        self,
        identity: ID,
        op: OP,
    ) -> <<Self as Edible>::SequentialIterator as Iterator>::Item
    where
        OP: Fn(
                <<Self as Edible>::SequentialIterator as Iterator>::Item,
                <<Self as Edible>::SequentialIterator as Iterator>::Item,
            ) -> <<Self as Edible>::SequentialIterator as Iterator>::Item
            + Sync
            + Send,
        ID: Fn() -> <<Self as Edible>::SequentialIterator as Iterator>::Item + Sync + Send,
    {
        schedule(self, &identity, &op)
    }
    // fn max(self) -> <<Self as Edible>::SequentialIterator as Iterator>::Item {
    //     self.iterator_map(|i| i.max()).reduce(|| None, |a, b| max(a, b))
    // }
}
