use crate::prelude::*;
use crate::scheduler::schedule_reduce;
use std::iter::successors;

pub trait FiniteParallelIterator: ParallelIterator {
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        let upper_bound = (self.len() as f64).sqrt().ceil() as usize;
        Box::new(successors(Some(1), move |s| {
            Some(std::cmp::min(s * 2, upper_bound))
        }))
    }
    /// Sums all content of the iterator.
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v = vec![1u32, 2, 3];
    /// assert_eq!(v.par_iter().sum::<u32>(), 6);
    /// assert_eq!((0u32..3).into_par_iter().sum::<u32>(), 3);
    /// ```
    fn sum<S>(self) -> S
    where
        S: Send + core::iter::Sum<S> + core::iter::Sum<Self::Item>,
    {
        //TODO: we are stuck with that until iterator_fold kicks in
        self.map(|e| std::iter::once(e).sum::<S>()).reduce(
            || std::iter::empty::<S>().sum::<S>(),
            |a, b| std::iter::once(a).chain(std::iter::once(b)).sum::<S>(),
        )
    }
    fn reduce<ID, OP>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let len = self.len();
        let i = self.borrow_on_left_for(len);
        schedule_reduce(i, &identity, &op, identity())
    }
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).reduce(|| (), |(), ()| ())
    }
    // here goes methods which cannot be applied to infinite iterators like sum
}

marked!(FiniteParallelIterator, ImplementsFiniteParallelIterator);
