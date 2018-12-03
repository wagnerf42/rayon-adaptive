use activated_input::ActivatedInput;
use folders::iterator_fold::{AdaptiveIteratorFold, IteratorFold};
use rayon::prelude::IndexedParallelIterator;
use std::iter;
use std::marker::PhantomData;
use {Divisible, DivisibleAtIndex};

pub trait IntoAdaptiveIterator: IntoIterator + DivisibleAtIndex {
    fn into_adapt_iter(self) -> Iter<Self> {
        Iter { input: self }
    }
}

impl<I: IntoIterator + DivisibleAtIndex> IntoAdaptiveIterator for I {}

pub struct Iter<I: IntoIterator + DivisibleAtIndex> {
    input: I,
}

impl<I: IntoIterator + DivisibleAtIndex> IntoIterator for Iter<I> {
    type Item = I::Item;
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.input.into_iter()
    }
}

impl<I: IntoIterator + DivisibleAtIndex> Divisible for Iter<I> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let (left, right) = self.input.split();
        (Iter { input: left }, Iter { input: right })
    }
}

impl<I: IntoIterator + DivisibleAtIndex> DivisibleAtIndex for Iter<I> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.input.split_at(index);
        (Iter { input: left }, Iter { input: right })
    }
}

pub struct Map<I: AdaptiveIterator, F> {
    base: I,
    map_op: F,
}

impl<IN, R: Send, I: AdaptiveIterator<Item = IN>, F: Fn(IN) -> R> IntoIterator for Map<I, F> {
    type Item = R;
    type IntoIter = iter::Map<I::IntoIter, F>;
    fn into_iter(self) -> Self::IntoIter {
        self.base.into_iter().map(self.map_op)
    }
}

impl<I: AdaptiveIterator, F: Send + Sync + Copy> Divisible for Map<I, F> {
    fn len(&self) -> usize {
        self.base.len()
    }
    fn split(self) -> (Self, Self) {
        let (left, right) = self.base.split();
        (
            Map {
                base: left,
                map_op: self.map_op,
            },
            Map {
                base: right,
                map_op: self.map_op,
            },
        )
    }
}

impl<I: AdaptiveIterator, F: Send + Sync + Copy> DivisibleAtIndex for Map<I, F> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.base.split_at(index);
        (
            Map {
                base: left,
                map_op: self.map_op,
            },
            Map {
                base: right,
                map_op: self.map_op,
            },
        )
    }
}

pub trait AdaptiveIterator: IntoIterator + DivisibleAtIndex {
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

impl<I: IntoIterator + DivisibleAtIndex> AdaptiveIterator for Iter<I> {}
impl<IN, R: Send, I: AdaptiveIterator<Item = IN>, F: Fn(IN) -> R + Send + Sync + Copy>
    AdaptiveIterator for Map<I, F>
{}

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
