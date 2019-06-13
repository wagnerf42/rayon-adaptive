use crate::prelude::*;
use crate::Policy;
use std::cmp::{max, min};
use std::iter;
use std::marker::PhantomData;
///
/// `Chain` is returned by the `chain` method on parallel iterators
///
///
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
        Some(
            (self.a.base_length().expect("Infinite iterator") as u64
                + self.a.base_length().expect("Infinite iterator") as u64) as usize,
        )
    }

    fn divide_at(self, index: usize) -> (Self, Self) {
        let len = self.a.base_length().expect("Infinite iterator");
        let (a1, a2) = self.a.divide_at(min(len, index));
        let (b1, b2) = self.b.divide_at(max(index - len, 0));
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
        let remaining_size = max(size - self.a.base_length().expect("Infinite Iterator"), 0);
        let size_a = min(size, self.a.base_length().expect("Infinite Iterator"));
        self.a
            .extract_iter(size_a)
            .chain(self.b.extract_iter(remaining_size))
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        self.a.to_sequential().chain(self.b.to_sequential())
    }

    fn policy(&self) -> Policy {
        match self.a.policy() {
            Policy::DefaultPolicy => self.b.policy(),
            _ => match self.b.policy() {
                Policy::DefaultPolicy => self.a.policy(),
                _ => panic!("Each iterator have a different policy"),
            },
        }
    }
}
