use super::{AdaptiveIterator, Divisible, DivisibleAtIndex};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Zip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A: AdaptiveIterator, B: AdaptiveIterator> IntoIterator for Zip<A, B> {
    type Item = (A::Item, B::Item);
    type IntoIter = iter::Zip<A::IntoIter, B::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.a.into_iter().zip(self.b.into_iter())
    }
}

impl<A: AdaptiveIterator, B: AdaptiveIterator> Divisible for Zip<A, B> {
    fn len(&self) -> usize {
        std::cmp::min(self.a.len(), self.b.len())
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.a.split();
        let (left_b, right_b) = self.b.split();
        (
            Zip {
                a: left_a,
                b: left_b,
            },
            Zip {
                a: right_a,
                b: right_b,
            },
        )
    }
}

impl<A: AdaptiveIterator, B: AdaptiveIterator> DivisibleAtIndex for Zip<A, B> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left_a, right_a) = self.a.split_at(index);
        let (left_b, right_b) = self.b.split_at(index);
        (
            Zip {
                a: left_a,
                b: left_b,
            },
            Zip {
                a: right_a,
                b: right_b,
            },
        )
    }
}

impl<A: AdaptiveIterator, B: AdaptiveIterator> AdaptiveIterator for Zip<A, B> {}
