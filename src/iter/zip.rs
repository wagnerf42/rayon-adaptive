use prelude::*;
use std;
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
    fn base_length(&self) -> usize {
        std::cmp::min(self.a.base_length(), self.b.base_length())
    }
    fn divide(self) -> (Self, Self) {
        let (left_a, right_a) = self.a.divide();
        let (left_b, right_b) = self.b.divide();
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

impl<A: AdaptiveIterator, B: AdaptiveIterator> DivisibleIntoBlocks for Zip<A, B> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left_a, right_a) = self.a.divide_at(index);
        let (left_b, right_b) = self.b.divide_at(index);
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

impl<A: AdaptiveIterator, B: AdaptiveIterator> DivisibleAtIndex for Zip<A, B> {}
impl<A: AdaptiveIterator, B: AdaptiveIterator> AdaptiveIterator for Zip<A, B> {}
impl<A: AdaptiveIndexedIterator, B: AdaptiveIndexedIterator> AdaptiveIndexedIterator for Zip<A, B> {}
