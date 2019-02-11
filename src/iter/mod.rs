use crate::activated_input::ActivatedInput;
use crate::folders::{fold::Fold, iterator_fold::AdaptiveIteratorFold};
use crate::prelude::*;
use crate::traits::{BlockedOrMore, BlockedPower};
use std::iter::Empty;
use std::marker::PhantomData;
pub mod map;
use self::map::Map;
pub mod iter;
use self::iter::Iter;
pub mod zip;
use self::zip::Zip;
mod cloned;
use self::cloned::Cloned;
mod filter;
use self::filter::Filter;
use crate::policy::ParametrizedInput;
use std;
use std::cmp::min;
mod collect;
pub use self::collect::{FromAdaptiveBlockedIterator, FromAdaptiveIndexedIterator};
pub(crate) mod hash;
pub(crate) mod str;
use crate::utils::powers;
use crate::utils::AbortingDivisible;
use std::sync::atomic::{AtomicBool, Ordering};

pub trait IntoAdaptiveIterator: IntoIterator + DivisibleIntoBlocks {
    fn into_adapt_iter(self) -> Iter<Self> {
        Iter { input: self }
    }
}

impl<I: IntoIterator + DivisibleIntoBlocks> IntoAdaptiveIterator for I {}

pub trait AdaptiveIterator: IntoIterator + DivisibleIntoBlocks {
    /// Creates an iterator which clones all of its elements.
    /// This is useful when you have an iterator over &T, but you need an iterator over T.
    fn cloned<'a, T: 'a>(self) -> Cloned<Self>
    where
        Self: AdaptiveIterator<Item = &'a T>,
    {
        Cloned { it: self }
    }
    //TODO: functions implement Copy but not clone ?????
    //what about Sync and Send ???
    fn filter<P: Fn(&Self::Item) -> bool + Clone + Sync + Send>(
        self,
        predicate: P,
    ) -> Filter<Self, P> {
        Filter {
            iter: self,
            predicate,
        }
    }
    fn map<R: Send, F: Fn(Self::Item) -> R + Send + Sync + Copy>(self, map_op: F) -> Map<Self, F> {
        Map { base: self, map_op }
    }
}

/// These iterators allow zipping, skipping and taking.
pub trait AdaptiveIndexedIterator: AdaptiveIterator + DivisibleAtIndex {
    /// Zip the two given iterators together.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v1 = vec![1u32; 1000];
    /// let v2 = vec![2u32; 1000];
    /// // let's compute the scalar product
    /// let s:u32 = v1.into_adapt_iter().zip(v2.into_adapt_iter()).map(|(x1, x2)| x1*x2).sum();
    /// assert_eq!(s, 2000);
    /// ```
    fn zip<U: AdaptiveIndexedIterator>(self, other: U) -> Zip<Self, U> {
        Zip { a: self, b: other }
    }
}

pub trait AdaptiveIteratorRunner<I: AdaptiveIterator, S: Iterator<Item = usize>>:
    AdaptiveRunner<I, S>
{
    fn find_any<P>(self, predicate: P) -> Option<I::Item>
    where
        P: Fn(&I::Item) -> bool + Sync + Send,
        I::Item: Sync + Send,
    {
        let found = AtomicBool::new(false);
        let (input, policy, sizes) = self.input_policy_sizes();
        let aborting_input = AbortingDivisible {
            real_content: input,
            abort: &found,
        };
        aborting_input
            .with_policy(policy)
            .by_blocks(sizes)
            .cutting_fold(
                || None,
                |f, i| {
                    if f.is_some() {
                        f
                    } else {
                        let new_f = i.into_iter().find(&predicate);
                        if new_f.is_some() {
                            found.store(true, Ordering::Relaxed)
                        }
                        new_f
                    }
                },
            )
            .into_iter()
            .filter_map(|o| o)
            .next()
    }

    /// Find first e in iterator such that predicate(e) is true.
    /// This implementation is efficient.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0..1000).into_adapt_iter().find_first(|&x| x == 100), Some(100));
    /// ```
    fn find_first<P>(self, predicate: P) -> Option<I::Item>
    where
        P: Fn(&I::Item) -> bool + Sync + Send,
        I::Item: Sync + Send,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        let len = input.base_length();
        let base_size = min((len as f64).log(2.0).ceil() as usize, len);
        input
            .with_policy(policy)
            .by_blocks(sizes.chain(powers(base_size)))
            .partial_fold(
                || None,
                |found, i, limit| {
                    //TODO: nothing is remaining if found.
                    //should we have options ???
                    let (todo, remaining) = i.divide_at(limit);
                    (
                        found.or_else(|| todo.into_iter().find(&predicate)),
                        remaining,
                    )
                },
            )
            .into_iter()
            .filter_map(|o| o)
            .next()
    }
    /// Return if any element e in the iterator is such that
    /// predicate(e) is true.
    /// This algorithm is work efficient and should produce speedups
    /// on fine grain instances.
    ///
    /// # Example
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert!((0..10_000).into_adapt_iter().any(|x| x == 2345))
    /// ```
    fn any<P>(self, predicate: P) -> bool
    where
        P: Fn(I::Item) -> bool + Sync + Send,
    {
        let predicate_ref = &predicate;
        !self.all(|x| !predicate_ref(x))
    }

    /// Return if all elements e in the iterator are such that
    /// predicate(e) is true.
    /// This algorithm is work efficient and should produce speedups
    /// on fine grain instances.
    ///
    /// # Example
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert!((0..10_000).into_adapt_iter().zip((0..10_000).into_adapt_iter()).all(|(x, y)| x == y))
    /// ```

    fn all<P>(self, predicate: P) -> bool
    where
        P: Fn(I::Item) -> bool + Sync + Send,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        let base_size = std::cmp::min(
            (input.base_length() as f64).log(2.0).ceil() as usize,
            input.base_length(),
        );
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || true,
                fold_op: |s: bool, i: I, limit: usize| {
                    let (todo, remaining) = i.divide_at(limit);
                    (
                        if s {
                            todo.into_iter().all(&predicate)
                        } else {
                            false
                        },
                        remaining,
                    )
                },
                phantom: PhantomData,
            },
            policy,
            sizes: sizes.chain(powers(base_size)), // this way if empty we take powers
            power: PhantomData,
        }
        .into_iter()
        .all(|b| b)
    }
    /// Counts the number of items in this adaptive iterator.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0..100).into_adapt_iter().filter(|&x| x %2 ==0).count(), 50);
    /// ```
    fn count(self) -> usize {
        self.fold(|| 0, |s, _| s + 1).reduce(|s1, s2| s1 + s2)
    }
    /// Computes the maximum of all the items in the iterator.
    /// If the iterator is empty, None is returned; otherwise, Some(max) is returned.
    /// Note that the order in which the items will be reduced is not specified,
    /// so if the Ord impl is not truly associative, then the results are not deterministic.
    ///
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0..1000).into_adapt_iter().max().unwrap(), 999)
    /// ```
    fn max(self) -> Option<I::Item>
    where
        I::Item: Ord + Send + Sync,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || None,
                fold_op: |previous_max, i: I, limit: usize| {
                    let (todo, remaining) = i.divide_at(limit);
                    let new_max = todo.into_iter().max();
                    (std::cmp::max(previous_max, new_max), remaining)
                },
                phantom: PhantomData,
            },
            policy,
            sizes,
            power: PhantomData,
        }
        .reduce(std::cmp::max)
    }
    fn sum<SUM>(self) -> SUM
    where
        SUM: std::iter::Sum<I::Item> + Send + Sync + std::ops::Add<Output = SUM>,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || None.into_iter().sum(),
                fold_op: |s: SUM, i: I, limit: usize| {
                    let (todo, remaining) = i.divide_at(limit);
                    let s2 = todo.into_iter().sum();
                    (s + s2, remaining)
                },
                phantom: PhantomData,
            },
            policy,
            sizes,
            power: PhantomData,
        }
        .reduce(|a, b| a + b)
    }

    /// Apply *op* on each element.
    ///
    /// Example:
    ///
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let mut v1 = vec![0; 1000];
    /// let v2: Vec<_> = (0..1000).collect();
    /// // let's copy v2 into v1
    /// v1.as_mut_slice().into_adapt_iter().zip(v2.into_adapt_iter()).for_each(|(x1, x2)| *x1 = *
    /// x2);
    /// assert_eq!(v1, v2);
    /// ```
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(I::Item) + Sync + Send,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || (),
                fold_op: |_, i: I, limit: usize| {
                    let (todo, remaining) = i.divide_at(limit);
                    todo.into_iter().for_each(&op);
                    ((), remaining)
                },
                phantom: PhantomData,
            },
            policy,
            sizes,
            power: PhantomData,
        }
        .reduce(|_, _| ())
    }

    fn fold<IO, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<AdaptiveIteratorFold<I, IO, ID, F>, S, BlockedOrMore>
    where
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Sync + Send + Clone,
        F: Fn(IO, I::Item) -> IO + Sync + Send + Clone,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        ActivatedInput {
            input,
            folder: AdaptiveIteratorFold {
                identity_op: identity,
                fold_op,
                phantom: PhantomData,
            },
            policy,
            sizes,
            power: PhantomData,
        }
    }
}

/// Specializations of AdaptiveIteratorRunner.
pub trait AdaptiveIndexedIteratorRunner<I: AdaptiveIndexedIterator, S: Iterator<Item = usize>>:
    AdaptiveRunner<I, S>
{
    /// Collect turn an `AdaptiveIterator` into a collection.
    /// As of now it is only implemented for `Vec`.
    /// Collecting comes with different algorithms for each Divisibility type
    /// (`Divisible`, `DivisibleIntoBlocks`, `DivisibleAtIndex`)
    /// This version is the `DivisibleAtIndex` version and will incur very little overhead.
    ///
    /// Example
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v:Vec<_> = (0..10_000).into_adapt_iter().map(|i| i+1).collect();
    /// let vseq:Vec<_> = (0..=10_000).skip(1).collect();
    /// assert_eq!(v, vseq)
    /// ```
    fn collect<C>(self) -> C
    where
        I::Item: Send,
        C: FromAdaptiveIndexedIterator<I::Item>,
    {
        FromAdaptiveIndexedIterator::from_adapt_iter(self)
    }
}
pub trait AdaptiveBlockedIteratorRunner<
    I: AdaptiveIterator<Power = BlockedPower>,
    S: Iterator<Item = usize>,
>: AdaptiveRunner<I, S>
{
    /// Collect turn an `AdaptiveIterator` into a collection.
    /// As of now it is only implemented for `Vec`.
    /// Collecting comes with different algorithms for each Divisibility type
    /// (`Divisible`, `DivisibleIntoBlocks`, `DivisibleAtIndex`)
    /// This version is the `DivisibleIntoBlocks` version and will incur very some overhead
    /// moving data twice.
    ///
    /// Example
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v:Vec<_> = (0..10_000).into_adapt_iter().filter(|&i| i%2 == 0).collect();
    /// let vseq:Vec<_> = (0..5_000).map(|i| i*2).collect();
    /// assert_eq!(v, vseq)
    /// ```

    fn collect<C>(self) -> C
    where
        I::Item: Send,
        C: FromAdaptiveBlockedIterator<I::Item>,
    {
        FromAdaptiveBlockedIterator::from_adapt_iter(self)
    }
}

impl<I: AdaptiveIterator, S: Iterator<Item = usize>> AdaptiveIteratorRunner<I, S>
    for ParametrizedInput<I, S>
{
}
impl<I: AdaptiveIterator> AdaptiveIteratorRunner<I, Empty<usize>> for I {}

impl<I: AdaptiveIndexedIterator, S: Iterator<Item = usize>> AdaptiveIndexedIteratorRunner<I, S>
    for ParametrizedInput<I, S>
{
}
impl<I: AdaptiveIndexedIterator> AdaptiveIndexedIteratorRunner<I, Empty<usize>> for I {}

impl<I: AdaptiveIterator<Power = BlockedPower>, S: Iterator<Item = usize>>
    AdaptiveBlockedIteratorRunner<I, S> for ParametrizedInput<I, S>
{
}
impl<I: AdaptiveIterator<Power = BlockedPower>> AdaptiveBlockedIteratorRunner<I, Empty<usize>>
    for I
{
}
