use super::{AdaptiveIterator, Divisible, DivisibleAtIndex};
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
    fn len(&self) -> usize {
        self.iter.len()
    }
    fn split(self) -> (Self, Self) {
        let (left, right) = self.iter.split();
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

impl<I: AdaptiveIterator, P: Send + Sync + Copy> DivisibleAtIndex for Filter<I, P> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.iter.split_at(index);
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
{}
