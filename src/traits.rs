//! This module contains all traits enabling us to express some parallelism.
use scheduling::{schedule, Policy};
use std;
use std::collections::LinkedList;

pub trait Divisible: Sized + Send {
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Return our length.
    fn len(&self) -> usize;
    /// Use this function when splitting generates a tangible work overhead.
    fn work<F, G, M>(self, work_function: F, output_function: G, policy: Policy) -> M
    where
        F: Fn(Self, usize) -> Self + Sync,
        G: Fn(Self) -> M + Sync,
        M: Mergeable,
    {
        schedule(self, &work_function, &output_function, policy)
    }
}

/// Some genericity to use only one scheduler for all different input types.
struct LocalWork<I: DivisibleAtIndex, M: Mergeable> {
    remaining_work: I,
    output: Option<M>,
}

impl<I: DivisibleAtIndex, M: Mergeable> Divisible for LocalWork<I, M> {
    fn split(self) -> (Self, Self) {
        let (left_work, right_work) = self.remaining_work.split();
        (
            LocalWork {
                remaining_work: left_work,
                output: self.output,
            },
            LocalWork {
                remaining_work: right_work,
                output: None,
            },
        )
    }
    fn len(&self) -> usize {
        self.remaining_work.len()
    }
}

pub trait DivisibleAtIndex: Divisible {
    /// Divide ourselves where requested.
    fn split_at(self, index: usize) -> (Self, Self);
    /// Easy api but use only when splitting generates no tangible work overhead.
    /// TODO: have the same with Mergeable ?
    fn map_reduce<F, M>(self, map_function: F) -> M
    where
        F: Fn(Self) -> M + Sync,
        M: Mergeable,
    {
        let full_work = LocalWork {
            remaining_work: self,
            output: None,
        };
        full_work.work(
            |w, limit| -> LocalWork<Self, M> {
                let (todo_now, remaining) = w.remaining_work.split_at(limit);
                let new_result = map_function(todo_now);

                LocalWork {
                    remaining_work: remaining,
                    output: if let Some(output) = w.output {
                        Some(output.fuse_with_policy(new_result, Policy::Sequential))
                    } else {
                        Some(new_result)
                    },
                }
            },
            |w| w.output.unwrap(),
            Policy::Adaptive(1000),
        )
    }
}

/// All outputs must implement this trait.
pub trait Mergeable: Sized + Send {
    /// Merge two outputs into one.
    fn fuse(self, other: Self) -> Self;
    /// Merge two outputs into one, the way we are told.
    fn fuse_with_policy(self, other: Self, _policy: Policy) -> Self {
        self.fuse(other)
    }
}

impl Mergeable for () {
    fn fuse(self, _other: Self) -> Self {
        ()
    }
}

impl<T: Send> Mergeable for LinkedList<T> {
    fn fuse(self, other: Self) -> Self {
        let mut left = self;
        let mut right = other; // TODO: change type of self and other ?
        left.append(&mut right);
        left
    }
}

impl<T: Send> Mergeable for Option<T> {
    fn fuse(self, other: Self) -> Self {
        self.or(other)
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
