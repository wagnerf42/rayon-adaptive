use crate::prelude::*;
use crate::scheduler::schedule_reduce;
use std::iter::successors;

pub trait FiniteParallelIterator: ParallelIterator {
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn bound_size(&self, size: usize) -> usize {
        std::cmp::min(self.len(), size) // this is the default for finite iterators
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        let upper_bound = (self.len() as f64).sqrt().ceil() as usize;
        Box::new(successors(Some(1), move |s| {
            Some(std::cmp::min(s * 2, upper_bound))
        }))
    }
    fn reduce<ID, OP>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let len = self.len();
        let i = self.borrow_on_left_for(len);
        schedule_reduce(i, &identity, &op)
    }
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).reduce(|| (), |(), ()| ())
    }
    // here goes methods which cannot be applied to infinite iterators like sum
}
