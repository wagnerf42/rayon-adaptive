use crate::prelude::*;
use derive_divisible::{Divisible, DivisibleIntoBlocks};
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks)]
#[power(I::Power)]
pub struct Iter<I: IntoIterator + DivisibleIntoBlocks> {
    pub(crate) input: I,
}

impl<I: IntoIterator + DivisibleIntoBlocks> IntoIterator for Iter<I> {
    type Item = I::Item;
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.input.into_iter()
    }
}

impl<I: IntoIterator + DivisibleAtIndex> DivisibleAtIndex for Iter<I> {}

impl<I: IntoIterator + DivisibleIntoBlocks> AdaptiveIterator for Iter<I> {}
impl<I: IntoIterator + DivisibleAtIndex> AdaptiveIndexedIterator for Iter<I> {}
