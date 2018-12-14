use crate::prelude::*;
use derive_divisible::{Divisible, DivisibleIntoBlocks};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks)]
#[power(I::Power)]
pub struct Cloned<I: AdaptiveIterator> {
    pub(crate) it: I,
}

//impl<I: AdaptiveIterator> Divisible for Cloned<I> {
//    type Power = I::Power;
//    fn base_length(&self) -> usize {
//        self.it.base_length()
//    }
//    fn divide(self) -> (Self, Self) {
//        let (left, right) = self.it.divide();
//        (Cloned { it: left }, Cloned { it: right })
//    }
//}

//impl<I: AdaptiveIterator> DivisibleIntoBlocks for Cloned<I> {
//    fn divide_at(self, index: usize) -> (Self, Self) {
//        let (left, right) = self.it.divide_at(index);
//        (Cloned { it: left }, Cloned { it: right })
//    }
//}

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
