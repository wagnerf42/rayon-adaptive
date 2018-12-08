use prelude::*;
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Map<I: AdaptiveIterator, F> {
    pub(crate) base: I,
    pub(crate) map_op: F,
}

impl<R: Send, I: AdaptiveIterator, F: Fn(I::Item) -> R> IntoIterator for Map<I, F> {
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

impl<I: AdaptiveIterator, F: Send + Sync + Copy> DivisibleIntoBlocks for Map<I, F> {
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
impl<I: AdaptiveIndexedIterator, F: Send + Sync + Copy> DivisibleAtIndex for Map<I, F> {}

impl<R: Send, I: AdaptiveIterator, F: Fn(I::Item) -> R + Send + Sync + Copy> AdaptiveIterator
    for Map<I, F>
{}
impl<R: Send, I: AdaptiveIndexedIterator, F: Fn(I::Item) -> R + Send + Sync + Copy>
    AdaptiveIndexedIterator for Map<I, F>
{}
