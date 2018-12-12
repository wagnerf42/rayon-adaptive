use super::{AdaptiveIterator, Divisible, DivisibleIntoBlocks};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Filter<I: AdaptiveIterator, P> {
    pub(crate) iter: I,
    pub(crate) predicate: P,
}

impl<I: AdaptiveIterator, P: Fn(&I::Item) -> bool> IntoIterator for Filter<I, P> {
    type Item = I::Item;
    type IntoIter = iter::Filter<I::IntoIter, P>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter.into_iter().filter(self.predicate)
    }
}

impl<I: AdaptiveIterator, P: Send + Sync + Copy> Divisible for Filter<I, P> {
    fn base_length(&self) -> usize {
        self.iter.base_length()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iter.divide();
        (
            Filter {
                iter: left,
                predicate: self.predicate,
            },
            Filter {
                iter: right,
                predicate: self.predicate,
            },
        )
    }
}

impl<I: AdaptiveIterator, P: Send + Sync + Copy> DivisibleIntoBlocks for Filter<I, P> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.divide_at(index);
        (
            Filter {
                iter: left,
                predicate: self.predicate,
            },
            Filter {
                iter: right,
                predicate: self.predicate,
            },
        )
    }
}

impl<I: AdaptiveIterator, P: Fn(&I::Item) -> bool + Send + Sync + Copy> AdaptiveIterator
    for Filter<I, P>
{
}
