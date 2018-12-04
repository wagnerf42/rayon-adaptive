use activated_input::ActivatedInput;
use folders::{fold::Fold, iterator_fold::AdaptiveIteratorFold};
use policy::AdaptiveRunner;
use std::marker::PhantomData;
use {Divisible, DivisibleAtIndex};
mod map;
use self::map::Map;
mod iter;
use self::iter::Iter;
mod zip;
use self::zip::Zip;
mod filter;
use self::filter::Filter;
use policy::ParametrizedInput;
use std;
use std::cmp::min;

pub trait IntoAdaptiveIterator: IntoIterator + DivisibleAtIndex {
    fn into_adapt_iter(self) -> Iter<Self> {
        Iter { input: self }
    }
}

impl<I: IntoIterator + DivisibleAtIndex> IntoAdaptiveIterator for I {}

pub trait AdaptiveIterator: IntoIterator + DivisibleAtIndex {
    fn filter<P: Fn(&Self::Item) -> bool>(self, predicate: P) -> Filter<Self, P> {
        Filter {
            iter: self,
            predicate,
        }
    }
    /// CAREFUL, THIS IS NOT SOUND YET
    fn zip<U: AdaptiveIterator>(self, other: U) -> Zip<Self, U> {
        Zip { a: self, b: other }
    }
    fn map<R: Send, F: Fn(Self::Item) -> R + Send + Sync + Copy>(self, map_op: F) -> Map<Self, F> {
        Map { base: self, map_op }
    }
}

fn powers(starting_value: usize) -> impl Iterator<Item = usize> {
    (0..).scan(starting_value, |state, _| {
        *state *= 2;
        Some(*state)
    })
}

pub trait AdaptiveIteratorRunner<I: AdaptiveIterator>: AdaptiveRunner<I> {
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
        let len = self.input_len();
        let base_size = min((len as f64).log(2.0).ceil() as usize, len);
        self.partial_fold(
            || None,
            |found, i, limit| {
                //TODO: nothing is remaining if found.
                //should we have options ???
                let (todo, remaining) = i.split_at(limit);
                (
                    found.or_else(|| todo.into_iter().find(&predicate)),
                    remaining,
                )
            },
        ).by_blocks(powers(base_size))
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
        let (input, policy) = self.input_and_policy();
        let base_size = std::cmp::min((input.len() as f64).log(2.0).ceil() as usize, input.len());
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || true,
                fold_op: |s: bool, i: I, limit: usize| {
                    let (todo, remaining) = i.split_at(limit);
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
        }.by_blocks(powers(base_size))
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
    fn sum<S>(self) -> S
    where
        S: std::iter::Sum<I::Item> + Send + Sync + std::ops::Add<Output = S>,
    {
        let (input, policy) = self.input_and_policy();
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || None.into_iter().sum(),
                fold_op: |s: S, i: I, limit: usize| {
                    let (todo, remaining) = i.split_at(limit);
                    let s2 = todo.into_iter().sum();
                    (s + s2, remaining)
                },
                phantom: PhantomData,
            },
            policy,
        }.reduce(|a, b| a + b)
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
        let (input, policy) = self.input_and_policy();
        ActivatedInput {
            input,
            folder: Fold {
                identity_op: || (),
                fold_op: |_, i: I, limit: usize| {
                    let (todo, remaining) = i.split_at(limit);
                    todo.into_iter().for_each(&op);
                    ((), remaining)
                },
                phantom: PhantomData,
            },
            policy,
        }.reduce(|_, _| ())
    }

    fn fold<IO, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<AdaptiveIteratorFold<I, IO, ID, F>>
    where
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Sync + Send + Clone,
        F: Fn(IO, I::Item) -> IO + Sync + Send + Clone,
    {
        let (input, policy) = self.input_and_policy();
        ActivatedInput {
            input,
            folder: AdaptiveIteratorFold {
                identity_op: identity,
                fold_op,
                phantom: PhantomData,
            },
            policy,
        }
    }
}

impl<I: AdaptiveIterator> AdaptiveIteratorRunner<I> for ParametrizedInput<I> {}
impl<I: AdaptiveIterator> AdaptiveIteratorRunner<I> for I {}
