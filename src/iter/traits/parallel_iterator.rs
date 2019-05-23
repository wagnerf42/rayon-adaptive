//! Iterator governing traits.
use crate::divisibility::{BasicPower, BlockedPower, BlockedPowerOrMore, IndexedPower};
use crate::iter::{ByBlocks, FilterMap, FlatMap, FlatMapSeq, Fold, IteratorFold, Map, WithPolicy};
use crate::prelude::*;
use crate::schedulers::schedule;
use crate::Policy;
use std::cmp::max;
use std::iter::{empty, successors};
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

    /// Parallel flat_map with a map to sequential iterators.
    fn flat_map_seq<F: Clone, PI>(self, map_op: F) -> FlatMapSeq<P, Self, F>
    where
        F: Fn(Self::Item) -> PI + Sync + Send,
        PI: IntoIterator,
        PI::Item: Send,
    {
        FlatMapSeq {
            base: self,
            map_op,
            phantom: PhantomData,
        }
    }

    /// Parallel flat_map with a map to parallel iterators.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((2u64..5).into_par_iter().flat_map(|i| (1..i)).collect::<Vec<_>>(), vec![1, 1, 2, 1, 2, 3])
    /// ```
    fn flat_map<F: Clone, INTO, PIN>(self, map_op: F) -> FlatMap<P, PIN, Self, INTO::Iter, F>
    where
        PIN: Power,
        F: Fn(Self::Item) -> INTO + Sync + Send,
        INTO: IntoParallelIterator<PIN>,
        INTO::Item: Send,
    {
        FlatMap::OuterIterator(self, map_op, Default::default())
    }

    /// Fold each sequential iterator into a single value.
    /// See the max method below as a use case.
    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<P, Self, F>
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
    /// assert_eq!((0u64..100).into_par_iter().max(), Some(99))
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
    /// assert_eq!(
    ///     (0u64..100).into_par_iter().fold(Vec::new, |mut v, i| {
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
    /// filter map
    fn filter_map<Predicate, R>(self, filter_op: Predicate) -> FilterMap<P, Self, Predicate>
    where
        Predicate: Fn(Self::Item) -> Option<R> + Sync + Send + Clone,
        R: Send,
    {
        FilterMap {
            base: self,
            filter_op,
            phantom: PhantomData,
        }
    }
    /// Map each element of the `ParallelIterator`.
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!(
    ///     (0u64..100).into_par_iter().map(|i| i*2).max(),
    ///     Some(198)
    /// )
    /// ```
    fn map<F, R>(self, map_op: F) -> Map<P, Self, F>
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

    /// Turn a parallel iterator into a collection.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((1u64..4).into_par_iter().collect::<Vec<_>>(), vec![1,2,3])
    /// ```
    fn collect<C: FromParallelIterator<Self::Item>>(self) -> C {
        C::from_par_iter(self)
    }
}

/// Here go all methods for basic power only.
pub trait BasicParallelIterator: ParallelIterator<BasicPower> {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

/// Here go all methods for blocked power only.
pub trait BlockedParallelIterator: ParallelIterator<BlockedPower> {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

//TODO: WE NEED A METHOD FOR COLLECT UP TO BLOCKED

/// Here go all methods for indexed.
pub trait IndexedParallelIterator: ParallelIterator<IndexedPower> {
    /// zip two iterators
    fn zip() {
        unimplemented!()
    }
    /// fast find, by blocks
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((1..).into_par_iter().find_first(|&x| x%10 == 0), Some(10));
    /// ```
    fn find_first<P>(self, predicate: P) -> Option<Self::Item>
    where
        P: Fn(&Self::Item) -> bool + Sync,
    {
        self.blocks(successors(Some(1), |p| Some(2 * p)))
            .map(|b| {
                b.iterator_fold(|mut i| i.find(&predicate))
                    .reduce(|| None, Option::or)
            })
            .filter_map(|o| o)
            .next()
    }
}

impl<I: ParallelIterator<IndexedPower>> IndexedParallelIterator for I {}
