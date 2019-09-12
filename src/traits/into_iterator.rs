use crate::prelude::*;

pub trait IntoParallelIterator {
    type Iter: ParallelIterator<Item = Self::Item>;
    type Item: Send;
    fn into_par_iter(self) -> Self::Iter;
}

impl<I: ParallelIterator> IntoParallelIterator for I {
    type Iter = Self;
    type Item = <Self as ItemProducer>::Item;
    fn into_par_iter(self) -> Self::Iter {
        self
    }
}
