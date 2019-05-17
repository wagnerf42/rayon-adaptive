//! Iterator governing traits.
use super::{ByBlocks, Fold, IteratorFold, Map, WithPolicy};
use crate::divisibility::{BasicPower, BlockedPower, IndexedPower};
use crate::prelude::*;
use crate::schedulers::schedule;
use crate::Policy;
use std::cmp::max;
use std::iter::empty;
use std::marker::PhantomData;

/// This traits enables to implement all basic methods for all type of iterators.
pub trait ParallelIterator<P: Power>: Divisible<P> + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send; // TODO: can we get rid of that and keep a short name ?
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator<Item = Self::Item>;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self);
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon(1)
    }

    /// Return an iterator on sizes of all macro blocks.
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        Box::new(empty())
    }
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
    /// Sets scheduling policy.
    fn with_policy(self, policy: Policy) -> WithPolicy<P, Self> {
        WithPolicy {
            policy,
            iterator: self,
            phantom: PhantomData,
        }
    }
    /// Sets the macro-blocks sizes.
    fn by_blocks<I: Iterator<Item = usize> + Send + 'static>(self, sizes: I) -> ByBlocks<P, Self> {
        ByBlocks {
            sizes_iterator: Some(Box::new(sizes)),
            iterator: self,
            phantom: PhantomData,
        }
    }
    /// Reduce with call to scheduler.
    fn reduce<OP, ID>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let policy = self.policy();
        let sizes = self.blocks_sizes();
        schedule(policy, &mut self.blocks(sizes), &identity, &op)
    }
    /// Return the max of all elements.
    ///
    /// # Example
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// use rayon_adaptive::Policy;
    /// assert_eq!((0u64..100).with_policy(Policy::Join(10)).max(), Some(99))
    /// ```
    fn max(self) -> Option<Self::Item>
    where
        Self::Item: Ord,
    {
        self.iterator_fold(Iterator::max).reduce(|| None, max)
    }
    /// Fold parallel iterator. Self will be split dynamically. Each part gets folded
    /// independantly. We get back a `ParallelIterator` on all results of all sequential folds.
    ///
    /// Let's see for example a manual vector creation (not optimized, don't do that).
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// use rayon_adaptive::Policy;
    /// assert_eq!(
    ///     (0u64..100).with_policy(Policy::Join(10)).fold(Vec::new, |mut v, i| {
    ///         v.push(i);
    ///         v
    ///     })
    ///         .reduce(Vec::new, |mut v1, v2| {
    ///             v1.extend(v2);
    ///             v1
    ///         }),
    ///     (0u64..100).collect::<Vec<u64>>()
    /// )
    /// ```
    fn fold<T, ID, F>(self, identity: ID, fold_op: F) -> Fold<P, Self, T, ID, F>
    where
        F: Fn(T, Self::Item) -> T + Sync + Send + Clone,
        ID: Fn() -> T + Sync + Send + Clone,
        T: Send,
    {
        Fold {
            remaining_input: self,
            current_output: Some(identity()),
            identity,
            fold_op,
            phantom: PhantomData,
        }
    }
    /// Map each element of the `ParallelIterator`.
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// use rayon_adaptive::Policy;
    /// assert_eq!(
    ///     (0u64..100).with_policy(Policy::Join(10)).map(|i| i*2).max(),
    ///     Some(198)
    /// )
    /// ```
    fn map<F, R>(self, map_op: F) -> Map<R, P, Self, F>
    where
        F: Fn(Self::Item) -> R + Sync + Send + Clone,
        R: Send,
    {
        Map {
            iter: self,
            f: map_op,
            phantom: PhantomData,
        }
    }
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

// TODO: we cannot do that. maybe derive it for every parallel iterator ?
// impl<I: ParallelIterator<IndexedPower>> IntoIterator for I {
//     type Item = I::Item;
//     type IntoIter = FlatMap<
//         BlocksIterator<IndexedPower, I, Box<Iterator<Item = usize>>>,
//         LinkedList<Vec<I::Item>>,
//         fn(I) -> LinkedList<Vec<I::Item>>,
//     >;
//     fn into_iter(self) -> Self::IntoIter {
//         let sizes = self.blocks_sizes();
//         self.blocks(sizes).flat_map(|b| {
//             b.fold(Vec::new, |mut v, e| {
//                 v.append(e);
//                 v
//             })
//             .map(|v| once(v).collect::<LinkedList<Vec<I::Item>>>())
//             .reduce(|mut l1, l2| {
//                 l1.append(&mut l2);
//                 l1
//             })
//         })
//     }
// }
