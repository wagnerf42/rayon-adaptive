use activated_input::ActivatedInput;
use folders::{
    fold::Fold,
    iterator_fold::{AdaptiveIteratorFold, IteratorFold},
};
use rayon::prelude::IndexedParallelIterator;
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
use policy::Policy;

pub trait IntoAdaptiveIterator: IntoIterator + DivisibleAtIndex {
    fn into_adapt_iter(self) -> Iter<Self> {
        Iter { input: self }
    }
}

impl<I: IntoIterator + DivisibleAtIndex> IntoAdaptiveIterator for I {}

fn powers(starting_value: usize) -> impl Iterator<Item = usize> {
    (0..).scan(starting_value, |state, _| {
        *state *= 2;
        Some(*state)
    })
}

pub trait AdaptiveIterator: IntoIterator + DivisibleAtIndex {
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
        P: Fn(Self::Item) -> bool + Sync + Send,
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
        P: Fn(Self::Item) -> bool + Sync + Send,
    {
        let base_size = std::cmp::min((self.len() as f64).log(2.0).ceil() as usize, self.len());
        ActivatedInput {
            input: self,
            folder: Fold {
                identity_op: || true,
                fold_op: |s: bool, i: Self, limit: usize| {
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
            policy: Default::default(),
        }.by_blocks(powers(base_size))
        .all(|b| b)
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
        OP: Fn(Self::Item) + Sync + Send,
    {
        iterator_for_each(self, op, Default::default())
    }

    fn sum<S>(self) -> S
    where
        S: std::iter::Sum<Self::Item> + Send + Sync + std::ops::Add<Output = S>,
    {
        iterator_sum(self, Default::default())
    }
    fn filter<P: Fn(Self::Item) -> bool>(self, predicate: P) -> Filter<Self, P> {
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
    fn fold<IO, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<AdaptiveIteratorFold<Self, IO, ID, F>>
    where
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Sync + Send + Clone,
        F: Fn(IO, Self::Item) -> IO + Sync + Send + Clone,
    {
        iterator_fold(self, identity, fold_op, Default::default())
    }
}

pub struct DivisibleIterator<I>
where
    I: IndexedParallelIterator + Clone + Sync,
{
    pub(crate) inner_iter: I,
    pub(crate) range: (usize, usize),
}

impl<I> Divisible for DivisibleIterator<I>
where
    I: IndexedParallelIterator + Clone + Sync,
{
    fn split(self) -> (Self, Self) {
        let left_iter = self.inner_iter.clone();
        let right_iter = self.inner_iter;
        (
            DivisibleIterator {
                inner_iter: left_iter,
                range: (self.range.0, (self.range.0 + self.range.1) / 2 as usize),
            },
            DivisibleIterator {
                inner_iter: right_iter,
                range: (
                    (self.range.1 + self.range.0) / 2 as usize + 1,
                    self.range.1 as usize,
                ),
            },
        )
    }

    fn len(&self) -> usize {
        if self.range.1 > self.range.0 {
            self.range.1 - self.range.0
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        self.range.1 == self.range.0
    }
}

pub trait AdaptiveFolder: IndexedParallelIterator {
    fn adaptive_fold<IO, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<IteratorFold<Self, IO, ID, F>>
    where
        Self: Sync + IndexedParallelIterator + Clone,
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Sync + Send + Clone,
        F: Fn(IO, Self::Item) -> IO + Sync + Send + Clone,
    {
        inner_adaptive_fold(self, identity, fold_op)
    }
}

fn inner_adaptive_fold<I, IO, ID, F>(
    iterator: I,
    identity: ID,
    fold_op: F,
) -> ActivatedInput<IteratorFold<I, IO, ID, F>>
where
    I: Sync + IndexedParallelIterator + Clone,
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Sync + Send + Clone,
    F: Fn(IO, I::Item) -> IO + Sync + Send + Clone,
{
    let range = (0, iterator.len());
    let divisible_input = DivisibleIterator {
        inner_iter: iterator,
        range,
    };
    let iter_fold = IteratorFold {
        identity_op: identity,
        fold_op,
        phantom: PhantomData,
    };
    ActivatedInput {
        input: divisible_input,
        folder: iter_fold,
        policy: Default::default(),
    }
}

impl<I> AdaptiveFolder for I where I: IndexedParallelIterator + Sync + Clone {}

pub(crate) fn iterator_sum<S, I: AdaptiveIterator>(iterator: I, policy: Policy) -> S
where
    S: std::iter::Sum<I::Item> + Send + Sync + std::ops::Add<Output = S>,
{
    ActivatedInput {
        input: iterator,
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

pub(crate) fn iterator_fold<I: AdaptiveIterator, IO, ID, F>(
    iterator: I,
    identity: ID,
    fold_op: F,
    policy: Policy,
) -> ActivatedInput<AdaptiveIteratorFold<I, IO, ID, F>>
where
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Sync + Send + Clone,
    F: Fn(IO, I::Item) -> IO + Sync + Send + Clone,
{
    ActivatedInput {
        input: iterator,
        folder: AdaptiveIteratorFold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        },
        policy,
    }
}

pub(crate) fn iterator_for_each<I, OP>(iter: I, op: OP, policy: Policy)
where
    I: AdaptiveIterator,
    OP: Fn(I::Item) + Sync + Send,
{
    ActivatedInput {
        input: iter,
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
