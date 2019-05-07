//! Iterator governing traits.
//! `Edible` allows for a step by step extraction of sequential work from parallel iterator.
use super::IteratorFold;
use crate::divisibility::{BasicPower, BlockedPower, IndexedPower};
use crate::prelude::*;
use crate::schedulers::schedule;
use std::marker::PhantomData;

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
    /// Fold each sequential iterator into a single value.
    /// See the max method below as a use case.
    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<R, P, Self, F>
    where
        R: Sized + Send,
        F: Fn(Self::SequentialIterator) -> R + Send + Clone,
    {
        IteratorFold {
            iterator: self,
            fold: fold_op,
            phantom: PhantomData,
        }
    }
    /// Reduce with call to scheduler.
    fn reduce<OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        schedule(self, &identity, &op)
    }
    // fn max(self) -> Option<Self::Item> {
    //     self.iterator_fold(|i| i.max()).reduce(|| None, |a, b| max(a, b))
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
