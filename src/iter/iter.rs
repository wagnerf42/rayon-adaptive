use super::{AdaptiveIterator, Divisible, DivisibleAtIndex};
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
pub struct Iter<I: IntoIterator + DivisibleAtIndex> {
    pub(crate) input: I,
}

impl<I: IntoIterator + DivisibleAtIndex> IntoIterator for Iter<I> {
    type Item = I::Item;
    type IntoIter = I::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.input.into_iter()
    }
}

impl<I: IntoIterator + DivisibleAtIndex> Divisible for Iter<I> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let (left, right) = self.input.split();
        (Iter { input: left }, Iter { input: right })
    }
}

impl<I: IntoIterator + DivisibleAtIndex> DivisibleAtIndex for Iter<I> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.input.split_at(index);
        (Iter { input: left }, Iter { input: right })
    }
}

impl<I: IntoIterator + DivisibleAtIndex> AdaptiveIterator for Iter<I> {}
