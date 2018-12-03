//! This module contains all traits enabling us to express some parallelism.
use scheduling::schedule;
use std;
use std::marker::PhantomData;
use std::ops::Range;
use std::ptr;

use activated_input::ActivatedInput;
use chunks::Chunks;
use folders::{fold::Fold, work_fold::WorkFold};
pub use iter::AdaptiveFolder;
use policy::ParametrizedInput;
use {Folder, Policy};

pub trait Divisible: Sized + Send + Sync {
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Return our length.
    fn len(&self) -> usize;
    /// Is there something left to do ?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn with_policy(self, policy: Policy) -> ParametrizedInput<Self> {
        ParametrizedInput {
            input: self,
            policy,
        }
    }
    fn work<WF: Fn(Self, usize) -> Self + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<WorkFold<Self, WF>> {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self,
            folder,
            policy: Default::default(),
        }
    }
    fn partial_fold<O, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<Fold<Self, O, ID, F>>
    where
        O: Send + Sync,
        ID: Fn() -> O + Sync,
        F: Fn(O, Self, usize) -> (O, Self) + Sync,
    {
        let folder = Fold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self,
            folder,
            policy: Default::default(),
        }
    }
    /// Easy api when we return no results.
    fn for_each<WF>(self, work_function: WF)
    where
        WF: Fn(Self, usize) -> Self + Sync,
    {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }.map(|_| ());
        let reduce = |_, _| ();
        schedule(self, &folder, &reduce, Default::default())
    }
}

pub trait DivisibleAtIndex: Divisible {
    /// Divide ourselves where requested.
    fn split_at(self, index: usize) -> (Self, Self);
    /// Divide ourselves keeping right part in self.
    /// Returns the left part.
    /// NB: this is useful for iterators creation.
    fn cut_left_at(&mut self, index: usize) -> Self {
        // there is a lot of unsafe going on here.
        // I think it's ok. rust uses the same trick for moving iterators (vecs for example)
        unsafe {
            let my_copy = ptr::read(self);
            let (left, right) = my_copy.split_at(index);
            let pointer_to_self = self as *mut Self;
            ptr::write(pointer_to_self, right);
            left
        }
    }
    /// Get a sequential iterator on chunks of Self of given sizes.
    fn chunks<S: Iterator<Item = usize>>(self, sizes: S) -> Chunks<Self, S> {
        Chunks {
            remaining: self,
            remaining_sizes: sizes,
        }
    }
    /// Easy api but use only when splitting generates no tangible work overhead.
    fn map_reduce<MF, RF, O>(self, map_function: MF, reduce_function: RF) -> O
    where
        MF: Fn(Self) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send + Sync,
    {
        let folder = Fold {
            identity_op: || None,
            fold_op: |o: Option<O>, i: Self, limit: usize| -> (Option<O>, Self) {
                let (todo_now, remaining) = i.split_at(limit);
                let new_result = map_function(todo_now);
                (
                    if let Some(output) = o {
                        Some(reduce_function(output, new_result))
                    } else {
                        Some(new_result)
                    },
                    remaining,
                )
            },
            phantom: PhantomData,
        }.map(|o| o.unwrap());

        schedule(
            self,
            &folder,
            &|left, right| reduce_function(left, right),
            Default::default(),
        )
    }
}

impl<'a, T: Sync> Divisible for &'a [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: Sync> DivisibleAtIndex for &'a [T] {
    fn split_at(self, index: usize) -> (Self, Self) {
        self.split_at(index)
    }
}

//TODO: I don't get why the compiler requires send here
impl<'a, T: 'a + Sync + Send> Divisible for &'a mut [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at_mut(mid)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleAtIndex for &'a mut [T] {
    fn split_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(index)
    }
}

//TODO: be more generic but it seems complex
impl Divisible for Range<usize> {
    fn len(&self) -> usize {
        ExactSizeIterator::len(self)
    }
    fn split(self) -> (Self, Self) {
        let mid = self.start + ExactSizeIterator::len(&self) / 2;
        (self.start..mid, mid..self.end)
    }
}

//TODO: be more generic but it seems complex
impl DivisibleAtIndex for Range<usize> {
    fn split_at(self, index: usize) -> (Self, Self) {
        (
            self.start..(self.start + index),
            (self.start + index)..self.end,
        )
    }
}

//TODO: macroize all that stuff ; even better : derive ?
impl<A: Divisible, B: Divisible> Divisible for (A, B) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), self.1.len())
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        ((left_a, left_b), (right_a, right_b))
    }
}

//TODO: macroize all that stuff ; even better : derive ?
impl<A: DivisibleAtIndex, B: DivisibleAtIndex> DivisibleAtIndex for (A, B) {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left_a, right_a) = self.0.split_at(index);
        let (left_b, right_b) = self.1.split_at(index);
        ((left_a, left_b), (right_a, right_b))
    }
}

impl<A: Divisible, B: Divisible, C: Divisible> Divisible for (A, B, C) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), std::cmp::min(self.1.len(), self.2.len()))
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        let (left_c, right_c) = self.2.split();
        ((left_a, left_b, left_c), (right_a, right_b, right_c))
    }
}
