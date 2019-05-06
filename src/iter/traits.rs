//! Iterator governing traits.
//! `Edible` allows for a step by step extraction of sequential work from parallel iterator.
use crate::divisibility::{BasicPower, BlockedPower, IndexedPower};
use crate::prelude::*;
use crate::schedulers::schedule;

/// We can produce sequential iterators to be eaten slowly.
pub trait Edible: Sized + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send; // TODO: can we get rid of that and keep a short name ?
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator<Item = Self::Item>;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator);
}

/// This traits enables to implement all basic methods for all type of iterators.
pub trait ParallelIterator<P: Power>: Divisible<P> + Edible {
    //fn iterator_map<F>(self, F: map_op) -> ParallelIterator<IteratorMap<Self>> {
    //    unimplemented!()
    //}
    /// Reduce with call to scheduler.
    fn reduce<OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        schedule(self, &identity, &op)
    }
    // fn max(self) -> <<Self as Edible>::SequentialIterator as Iterator>::Item {
    //     self.iterator_map(|i| i.max()).reduce(|| None, |a, b| max(a, b))
    // }
}

/// Here go all methods for basic power only.
pub trait BasicParallelIterator: ParallelIterator<BasicPower> {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

//TODO: WE NEED A METHOD FOR COLLECT UP TO BLOCKED

/// Here go all methods for blocked or more.
pub trait BlockedParallelIterator: ParallelIterator<BlockedPower> {
    /// fast find
    fn find(self) {
        unimplemented!()
    }
}

/// Here go all methods for indexed.
pub trait IndexedParallelIterator: ParallelIterator<IndexedPower> {
    /// zip two iterators
    fn zip() {
        unimplemented!()
    }
}

impl<P: Power, I: Edible + Divisible<P>> ParallelIterator<P> for I {}
