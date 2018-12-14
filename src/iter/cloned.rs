use crate::prelude::*;
use derive_divisible::{Divisible, DivisibleIntoBlocks};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks)]
#[power(I::Power)]
pub struct Cloned<I: AdaptiveIterator> {
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

impl<I: AdaptiveIndexedIterator> DivisibleAtIndex for Cloned<I> {}

impl<'a, T: Clone + 'a, I: AdaptiveIterator<Item = &'a T>> AdaptiveIterator for Cloned<I> {}
impl<'a, T: Clone + 'a, I: AdaptiveIndexedIterator<Item = &'a T>> AdaptiveIndexedIterator
    for Cloned<I>
{
}
