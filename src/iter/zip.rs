use crate::prelude::*;
use crate::traits::IndexedPower;
use derive_divisible::{Divisible, DivisibleAtIndex, DivisibleIntoBlocks};
use std;
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks, DivisibleAtIndex)]
#[power(IndexedPower)]
pub struct Zip<A: AdaptiveIterator, B: AdaptiveIterator> {
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

impl<A: AdaptiveIterator, B: AdaptiveIterator> AdaptiveIterator for Zip<A, B> {}
impl<A: AdaptiveIndexedIterator, B: AdaptiveIndexedIterator> AdaptiveIndexedIterator for Zip<A, B> {}
