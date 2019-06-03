//! Iterator governing traits.
use crate::divisibility::{BasicPower, BlockedPower, BlockedPowerOrMore, IndexedPower};
use crate::help::{Help, Retriever};
use crate::iter::{ByBlocks, FilterMap, FlatMap, FlatMapSeq, Fold, IteratorFold, Map, WithPolicy, Zip};
use crate::prelude::*;
use crate::schedulers::schedule;
use crate::Policy;
use std::cmp::max;
use std::iter;
use std::iter::{empty, successors};
use std::marker::PhantomData;

/// This traits enables to implement all basic methods for all type of iterators.
pub trait ParallelIterator: Divisible + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send;
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
    fn flat_map_seq<F: Clone, PI>(self, map_op: F) -> FlatMapSeq<Self, F>
    where
        F: Fn(Self::Item) -> PI + Sync + Send,
        PI: IntoIterator,
        PI::Item: Send,
    {
        FlatMapSeq { base: self, map_op }
    }

    /// Parallel flat_map with a map to parallel iterators.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((2u64..5).into_par_iter().flat_map(|i| (1..i)).collect::<Vec<_>>(), vec![1, 1, 2, 1, 2, 3])
    /// ```
    fn flat_map<F: Clone, INTO>(self, map_op: F) -> FlatMap<Self, INTO::Iter, F>
    where
        F: Fn(Self::Item) -> INTO + Sync + Send,
        INTO: IntoParallelIterator,
        INTO::Item: Send,
    {
        FlatMap::OuterIterator(self, map_op)
    }

    /// Fold each sequential iterator into a single value.
    /// See the max method below as a use case.
    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<Self, F>
    where
        R: Sized + Send,
        F: Fn(Self::SequentialIterator) -> R + Send + Clone,
    {
        IteratorFold {
            iterator: self,
            fold: fold_op,
        }
    }
    /// Sets scheduling policy.
    fn with_policy(self, policy: Policy) -> WithPolicy<Self> {
        WithPolicy {
            policy,
            iterator: self,
        }
    }
    /// Sets the macro-blocks sizes.
    fn by_blocks<I: Iterator<Item = usize> + Send + 'static>(self, sizes: I) -> ByBlocks<Self> {
        ByBlocks {
            sizes_iterator: Some(Box::new(sizes)),
            iterator: self,
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
    fn fold<T, ID, F>(self, identity: ID, fold_op: F) -> Fold<Self, T, ID, F>
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
        }
    }
    /// filter map
    fn filter_map<Predicate, R>(self, filter_op: Predicate) -> FilterMap<Self, Predicate>
    where
        Predicate: Fn(Self::Item) -> Option<R> + Sync + Send + Clone,
        R: Send,
    {
        FilterMap {
            base: self,
            filter_op,
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
    fn map<F, R>(self, map_op: F) -> Map<Self, F>
    where
        F: Fn(Self::Item) -> R + Sync + Send + Clone,
        R: Send,
    {
        Map {
            iter: self,
            f: map_op,
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

    /// Apply closure to each element.
    ///
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let mut v = vec![0, 1, 1, 0, 0];
    /// v.as_mut_slice().into_par_iter().for_each(|e| *e = 1 - *e);
    /// assert_eq!(v, vec![1, 0, 0, 1, 1])
    /// ```
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync,
    {
        self.map(|e| {
            op(e);
        })
        .reduce(|| (), |_, _| ())
    }
}

/// Here go all methods for basic power only.
pub trait BasicParallelIterator: ParallelIterator {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

/// Here go all methods for blocked power only.
pub trait BlockedParallelIterator: ParallelIterator {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
}

/// Here go all methods for indexed.
pub trait IndexedParallelIterator: ParallelIterator {
    /// zip two iterators
    /// 
    /// Example:
    /// 
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let mut v = vec![0u64; 10_000];
    /// v.as_mut_slice().into_par_iter().zip((0..10_000u64).into_par_iter()).for_each(|(o, e)| *o = e);
    /// assert_eq!(v, (0..10_000).collect::<Vec<u64>>());
    /// ```
    fn zip<Z>(self, zip_op: Z) -> Zip<Self, Z::Iter> where
    Z: IntoParallelIterator,
    Z::Iter: ParallelIterator<Power=IndexedPower>, {
        Zip {
            a: self,
            b: zip_op.into_par_iter(),
        }
    }
}

/// Here go all methods specialized for iterators which have a power at least Blocked.
pub trait BlockedOrMoreParallelIterator: ParallelIterator {
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

    /// Fully adaptive algorithms where one sequential worker is helped by other threads.
    fn with_help<C, H>(self, help_op: H) -> Help<Self, C>
    where
        C: Send,
        H: Fn(iter::Flatten<Retriever<Self, C>>) -> C + Sync + 'static, // TODO how bad is this 'static ?
    {
        Help {
            iterator: self,
            help_op: Box::new(help_op),
            phantom: PhantomData,
        }
    }
}

impl<I: ParallelIterator<Power = BasicPower>> BasicParallelIterator for I {}
impl<I: ParallelIterator<Power = BlockedPower>> BlockedParallelIterator for I {}
impl<I: ParallelIterator<Power = IndexedPower>> IndexedParallelIterator for I {}
impl<P: BlockedPowerOrMore, I: ParallelIterator<Power = P>> BlockedOrMoreParallelIterator for I {}
