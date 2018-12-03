use activated_input::ActivatedInput;
use folders::iterator_fold::{AdaptiveIteratorFold, IteratorFold};
use rayon::prelude::IndexedParallelIterator;
use std::marker::PhantomData;
use {Divisible, DivisibleAtIndex};
mod map;
use self::map::Map;
mod iter;
use self::iter::Iter;
mod zip;
use self::zip::Zip;

pub trait IntoAdaptiveIterator: IntoIterator + DivisibleAtIndex {
    fn into_adapt_iter(self) -> Iter<Self> {
        Iter { input: self }
    }
}

impl<I: IntoIterator + DivisibleAtIndex> IntoAdaptiveIterator for I {}

pub trait AdaptiveIterator: IntoIterator + DivisibleAtIndex {
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
        ActivatedInput {
            input: self,
            folder: AdaptiveIteratorFold {
                identity_op: identity,
                fold_op,
                phantom: PhantomData,
            },
            policy: Default::default(),
        }
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
