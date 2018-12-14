use super::{AdaptiveIterator, Divisible, DivisibleIntoBlocks};
use crate::traits::BlockedPower;
use derive_divisible::{Divisible, DivisibleIntoBlocks};
use std::iter;

#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Divisible, DivisibleIntoBlocks)]
#[power(BlockedPower)]
pub struct Filter<I: AdaptiveIterator, P: Clone + Send + Sync> {
    pub(crate) iter: I,
    #[divide_by(clone)]
    pub(crate) predicate: P,
}

impl<I: AdaptiveIterator, P: Fn(&I::Item) -> bool + Clone + Send + Sync> IntoIterator
    for Filter<I, P>
{
    type Item = I::Item;
    type IntoIter = iter::Filter<I::IntoIter, P>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter.into_iter().filter(self.predicate)
    }
}

impl<I: AdaptiveIterator, P: Fn(&I::Item) -> bool + Send + Sync + Copy> AdaptiveIterator
    for Filter<I, P>
{
}
