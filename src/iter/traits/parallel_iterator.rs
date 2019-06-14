//! Iterator governing traits.
use crate::divisibility::{BasicPower, BlockedPower, BlockedPowerOrMore, IndexedPower, Power};
use crate::help::{Help, Retriever};
use crate::iter::Chain;
use crate::iter::Try;
use crate::iter::{
    ByBlocks, Cap, Filter, FilterMap, FlatMap, FlatMapSeq, Fold, Interruptible, IteratorFold, Map,
    WithPolicy, Zip,
};
use crate::prelude::*;
use crate::schedulers::schedule;
use crate::schedulers_interruptible::schedule_interruptible;
use crate::Policy;
use std::cmp::{max, min};
use std::f32;
use std::iter;
use std::iter::{empty, once, successors, Sum};
use std::marker::PhantomData;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;

/// This traits enables to implement all basic methods for all type of iterators.
pub trait ParallelIterator: Divisible + Send {
    /// This registers the type of output produced (it IS the item of the SequentialIterator).
    type Item: Send;
    /// This registers the type of iterators produced.
    type SequentialIterator: Iterator<Item = Self::Item>;
    /// Convert ourselves to a standard sequential iterator.
    fn to_sequential(self) -> Self::SequentialIterator;
    /// Give us a sequential iterator corresponding to `size` iterations.
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator;
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::DefaultPolicy
    }

    /// Return an iterator on sizes of all macro blocks.
    fn blocks_sizes(&mut self) -> Box<Iterator<Item = usize>> {
        Box::new(empty())
    }

    /// Filter iterator with given closure.
    /// If your power was indexed it's not the case anymore.
    fn filter<P>(self, filter_op: P) -> Filter<Self, P>
    where
        P: Fn(&Self::Item) -> bool + Sync + Send,
    {
        Filter {
            base: self,
            filter_op,
        }
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

    ///
    /// Reduces the items in the iterator into one item using a fallible op.
    /// call to sechuler interruptible
    ///
    fn try_reduce<T, OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(T, T) -> Self::Item + Sync + Send,
        ID: Fn() -> T + Sync + Send,
        Self::Item: Try<Ok = T>,
    {
        let policy = self.policy();
        schedule_interruptible(policy, self, &identity, &op)
    }

    /// Get a sequential iterator on items produced in parallel.
    /// This iterator has the added bonus to be lazy on the blocks
    /// which means it will not consume more blocks than the strict minimum.
    fn reduced_iter(
        mut self,
    ) -> std::iter::FlatMap<
        crate::divisibility::BlocksIterator<Self, Box<Iterator<Item = usize>>>,
        std::iter::Flatten<std::collections::linked_list::IntoIter<Vec<Self::Item>>>,
        fn(Self) -> std::iter::Flatten<std::collections::linked_list::IntoIter<Vec<Self::Item>>>,
    > {
        let sizes = self.blocks_sizes();
        self.blocks(sizes).flat_map(|b| {
            b.fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .map(|v| std::iter::once(v).collect::<std::collections::LinkedList<Vec<Self::Item>>>())
            .reduce(std::collections::LinkedList::new, |mut l1, mut l2| {
                l1.append(&mut l2);
                l1
            })
            .into_iter()
            .flatten()
        })
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
    /// Cap the number of threads executing us to given number.
    /// We will automatically switch to an adaptive scheduling policy.
    /// TODO: small pb right now, if we iterate by blocks we get one
    /// less thread ?
    /// TODO: fix that by always iterating by blocks and adding one to the limit.
    ///
    /// Example:
    /// ```
    /// // let's cap to two threads and manually check we never get more.
    /// use rayon_adaptive::prelude::*;
    /// use rayon_adaptive::Policy;
    /// use std::sync::atomic::{AtomicUsize, Ordering};
    /// let count = AtomicUsize::new(0);
    /// (0..10_000u64).into_par_iter().with_policy(Policy::Adaptive(1000, 50_000)).cap(2).for_each(|_| {
    ///     let other_threads = count.fetch_add(1, Ordering::SeqCst);
    ///     assert!(other_threads < 2);
    ///     count.fetch_sub(1, Ordering::SeqCst);
    /// })
    /// ```
    fn cap(self, limit: usize) -> Cap<Self> {
        Cap {
            iterator: self,
            count: Arc::new(AtomicUsize::new(0)),
            limit,
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
    /// Tests if every elements matches a predicate
    ///
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert!((1u64..25).into_par_iter().all(|x| x > 0));
    /// assert!(!(0u64..25).into_par_iter().all(|x| x > 2));
    /// ```
    fn all<F>(mut self, f: F) -> bool
    where
        F: Fn(Self::Item) -> bool + Sync,
    {
        let size_entry = self.base_length().unwrap();
        let p = rayon::current_num_threads();
        let sizes_block = self.blocks_sizes();
        let mut policy = self.policy();
        policy = match policy {
            Policy::DefaultPolicy => {
                // if the user did not explicitely choose a scheduling policy we are going to choose one for him.
                // this is worthwhile here because this algorithm benefits from adaptive policies.
                if (((p as f64).log(2.0).ceil() as usize) * p * p * 100) < size_entry {
                    policy = Policy::Adaptive(((size_entry as f32).log2() * 2.0) as usize, 10_000);
                    policy
                } else {
                    policy // the size is too small for adaptive algorithms which work a little BEFORE dividing.
                }
            }
            _ => policy,
        };

        self.with_policy(policy)
            .blocks(sizes_block.chain(successors(Some(10_000usize * 2), |n| n.checked_mul(2))))
            .all(|block| {
                let b = AtomicBool::new(true);
                let i = Interruptible {
                    iterator: block,
                    keepexec: &b,
                };
                i.iterator_fold(|mut i| {
                    if !(i.all(&f)) {
                        b.store(false, Ordering::Relaxed);
                    }
                })
                .reduce(|| (), |_, _| ());
                b.load(Ordering::Relaxed)
            })
    }
    ///
    /// Test Function for all using schecule with boolean
    ///
    fn allscheduling<F>(self, f: F) -> bool
    where
        F: Fn(Self::Item) -> bool + Sync,
    {
        let f_ref = &f;
        match self
            .iterator_fold(|mut i| if i.all(f_ref) { Ok(()) } else { Err(()) })
            .try_reduce(|| (), |_, _| Ok(()))
        {
            // TODO
            Ok(_) => true,
            Err(_) => false,
        }
    }

    ///
    ///  Applies this closure to each element of the iterator, and if any of them return true, then so does any().
    /// If they all return false, it returns false.
    ///  Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert!((1u64..25).into_par_iter().any(|x| x > 0));
    /// assert!(!(0u64..25).into_par_iter().any(|x| x > 26));
    /// ```
    fn any<F>(self, f: F) -> bool
    where
        F: Fn(Self::Item) -> bool + Sync,
    {
        !self.all(|i| !f(i))
    }

    ///
    /// Sums up the items in the iterator.
    /// Note that the order in items will be reduced is not specified, so if the + operator is not truly associative
    ///
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((1u64..25).into_par_iter().sum::<u64>(),25u64*24 / 2);
    /// ```
    fn sum<S>(self) -> S
    where
        S: Send + Sum<Self::Item> + Sum<S>,
    {
        self.iterator_fold(|i| i.sum()).reduce(
            || empty::<S>().sum(),
            |s1, s2| once(s1).chain(once(s2)).sum(),
        )
    }
}

/// Here go all methods for basic power only.
pub trait BasicParallelIterator: ParallelIterator {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
    /// chain two iterators
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v = (0..5u64).into_par_iter().chain((0..10u64).into_par_iter()).sum::<u64>();
    /// assert_eq!(55,v);
    /// ```
    fn chain<C>(self, chain: C) -> Chain<Self, C::Iter, BasicPower>
    where
        C: IntoParallelIterator<Item = Self::Item>,
    {
        Chain {
            a: self,
            b: chain.into_par_iter(),
            p: Default::default(),
        }
    }
}

/// Here go all methods for blocked power only.
pub trait BlockedParallelIterator: ParallelIterator {
    /// slow find
    fn find(self) {
        unimplemented!()
    }
    /// chain two iterators
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v = (0..5u64).into_par_iter().chain((0..10u64).into_par_iter()).sum::<u64>();
    /// assert_eq!(55,v);
    /// ```
    fn chain<C>(
        self,
        chain: C,
    ) -> Chain<
        Self,
        C::Iter,
        <<<C as IntoParallelIterator>::Iter as Divisible>::Power as Power>::NotIndexed,
    >
    where
        C: IntoParallelIterator<Item = Self::Item>,
    {
        Chain {
            a: self,
            b: chain.into_par_iter(),
            p: Default::default(),
        }
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
    fn zip<Z>(self, zip_op: Z) -> Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Iter: ParallelIterator<Power = IndexedPower>,
    {
        Zip {
            a: self,
            b: zip_op.into_par_iter(),
        }
    }
    /// chain two iterators
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v = (0..5u64).into_par_iter().chain((0..10u64).into_par_iter()).sum::<u64>();
    /// assert_eq!(55,v);
    /// ```
    fn chain<C>(
        self,
        chain: C,
    ) -> Chain<Self, C::Iter, <<C as IntoParallelIterator>::Iter as Divisible>::Power>
    where
        C: IntoParallelIterator<Item = Self::Item>,
    {
        Chain {
            a: self,
            b: chain.into_par_iter(),
            p: Default::default(),
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
