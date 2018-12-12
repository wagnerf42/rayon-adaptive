use crate::prelude::*;
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
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

impl<I: IntoIterator + DivisibleIntoBlocks> Divisible for Iter<I> {
    fn base_length(&self) -> usize {
        self.input.base_length()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.input.divide();
        (Iter { input: left }, Iter { input: right })
    }
}

impl<I: IntoIterator + DivisibleIntoBlocks> DivisibleIntoBlocks for Iter<I> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.input.divide_at(index);
        (Iter { input: left }, Iter { input: right })
    }
}

impl<I: IntoIterator + DivisibleAtIndex> DivisibleAtIndex for Iter<I> {}

impl<I: IntoIterator + DivisibleIntoBlocks> AdaptiveIterator for Iter<I> {}
impl<I: IntoIterator + DivisibleAtIndex> AdaptiveIndexedIterator for Iter<I> {}
