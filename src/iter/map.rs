use crate::prelude::*;
use derive_divisible::{Divisible, DivisibleIntoBlocks};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks)]
pub struct Map<I: AdaptiveIterator, F: Clone + Send + Sync> {
    pub(crate) base: I,
    #[divide_by(clone)]
    pub(crate) map_op: F,
}

impl<R: Send, I: AdaptiveIterator, F: Fn(I::Item) -> R + Clone + Send + Sync> IntoIterator
    for Map<I, F>
{
    type Item = R;
    type IntoIter = iter::Map<I::IntoIter, F>;
    fn into_iter(self) -> Self::IntoIter {
        self.base.into_iter().map(self.map_op)
    }
}

impl<I: AdaptiveIndexedIterator, F: Send + Sync + Clone> DivisibleAtIndex for Map<I, F> {}

impl<R: Send, I: AdaptiveIterator, F: Fn(I::Item) -> R + Send + Sync + Copy> AdaptiveIterator
    for Map<I, F>
{
}
impl<R: Send, I: AdaptiveIndexedIterator, F: Fn(I::Item) -> R + Send + Sync + Copy>
    AdaptiveIndexedIterator for Map<I, F>
{
}
