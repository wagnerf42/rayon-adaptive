//! `chain` implementation.
use crate::prelude::*;
use crate::Policy;
use std::cmp::{max, min};
use std::iter;
use std::marker::PhantomData;
/// `Chain` is returned by the `chain` method on parallel iterators
pub struct Chain<A, B, P>
where
    A: ParallelIterator,
    B: ParallelIterator<Item = A::Item>,
    P: Power,
{
    pub(crate) a: A,
    pub(crate) b: B,
    pub(crate) p: PhantomData<P>,
}

impl<A, B, P> Divisible for Chain<A, B, P>
where
    A: ParallelIterator,
    B: ParallelIterator<Item = A::Item>,
    P: Power,
{
    type Power = P;
    fn base_length(&self) -> Option<usize> {
        // `a` cannot be infinite but `b` can.
        self.b.base_length().map(|b_len| {
            b_len
                + self
                    .a
                    .base_length()
                    .expect("first chained iterator is not finite")
        })
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        // we divide both `a` and `b` so that all sequential
        // iterators are of the same `Chain` type.
        let len = self.a.base_length().expect("Infinite iterator");
        let (a1, a2) = self.a.divide_at(min(len, index));
        let id = if len > index { 0 } else { index - len };
        let (b1, b2) = self.b.divide_at(max(id, 0));
        (
            Chain {
                a: a1,
                b: b1,
                p: Default::default(),
            },
            Chain {
                a: a2,
                b: b2,
                p: Default::default(),
            },
        )
    }
}

impl<A, B, P> ParallelIterator for Chain<A, B, P>
where
    A: ParallelIterator,
    B: ParallelIterator<Item = A::Item>,
    P: Power,
{
    type Item = A::Item;
    type SequentialIterator = iter::Chain<A::SequentialIterator, B::SequentialIterator>;

    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let len_a = self.a.base_length().expect("Infinite Iterator");
        let size_needed_in_b = max(size - len_a, 0);
        let size_needed_in_a = min(size, len_a);
        self.a
            .extract_iter(size_needed_in_a)
            .chain(self.b.extract_iter(size_needed_in_b))
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        self.a.to_sequential().chain(self.b.to_sequential())
    }

    fn policy(&self) -> Policy {
        match self.a.policy() {
            Policy::DefaultPolicy => self.b.policy(),
            _ => match self.b.policy() {
                Policy::DefaultPolicy => self.a.policy(),
                _ => panic!("Chained iterators have a different policy"),
            },
        }
    }
}
