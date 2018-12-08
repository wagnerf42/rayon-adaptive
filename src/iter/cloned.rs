use prelude::*;
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Cloned<I> {
    pub(crate) it: I,
}

impl<'a, I, T> IntoIterator for Cloned<I>
where
    I: AdaptiveIterator<Item = &'a T>,
    T: Clone + 'a,
{
    type Item = T;
    type IntoIter = iter::Cloned<I::IntoIter>;
    fn into_iter(self) -> Self::IntoIter {
        self.it.into_iter().cloned()
    }
}

impl<I: AdaptiveIterator> Divisible for Cloned<I> {
    fn len(&self) -> usize {
        self.it.len()
    }
    fn split(self) -> (Self, Self) {
        let (left, right) = self.it.split();
        (Cloned { it: left }, Cloned { it: right })
    }
}

impl<I: AdaptiveIterator> DivisibleIntoBlocks for Cloned<I> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.it.split_at(index);
        (Cloned { it: left }, Cloned { it: right })
    }
}
impl<I: AdaptiveIndexedIterator> DivisibleAtIndex for Cloned<I> {}

impl<'a, T: Clone + 'a, I: AdaptiveIterator<Item = &'a T>> AdaptiveIterator for Cloned<I> {}
impl<'a, T: Clone + 'a, I: AdaptiveIndexedIterator<Item = &'a T>> AdaptiveIndexedIterator
    for Cloned<I>
{}
