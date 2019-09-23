use crate::scheduler::schedule_reduce;
mod divisible;
// mod finite_parallel_iterator;
mod indexed;
mod into_iterator;
mod into_parallel_ref;
// mod parallel_iterator;
mod types;

pub use divisible::Divisible;
// pub use finite_parallel_iterator::FiniteParallelIterator;
pub use indexed::IndexedParallelIterator;
pub use into_iterator::IntoParallelIterator;
pub use into_parallel_ref::IntoParallelRefIterator;
// pub use parallel_iterator::ParallelIterator;
pub use types::{Indexed, ItemProducer, ParBorrowed, Powered, SeqBorrowed, Standard};

use crate::iter::{Filter, Map};
pub trait ParallelIterator: Powered + Sized
where
    Self: for<'e> ParBorrowed<'e>,
{
    /// Takes the number of iterations requested by the user
    /// and return the number we can really process.
    fn bound_iterations_number(&self, size: usize) -> usize;
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter;

    /// filter.
    /// # Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0u32..10).into_par_iter().filter(|&e| e%2==0).sum::<u32>(), 20)
    /// ```
    fn filter<P>(self, filter_op: P) -> Filter<Self, P>
    where
        P: Fn(&Self::Item) -> bool + Sync,
    {
        Filter {
            iterator: self,
            filter_op,
        }
    }

    fn map<F, R>(self, op: F) -> Map<Self, F>
    where
        R: Send,
        F: Fn(Self::Item) -> R + Send,
    {
        Map { op, base: self }
    }
    fn reduce<ID, OP>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let size = self.bound_iterations_number(std::usize::MAX);
        let single_block = self.par_borrow(size);
        single_block.block_reduce(&identity, &op)
    }

    /// Sums all content of the iterator.
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
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

    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).reduce(|| (), |(), ()| ())
    }
}

pub trait BorrowingParallelIterator: Divisible + ItemProducer + Send
where
    Self: for<'e> SeqBorrowed<'e>,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter;
    /// Return the number of iterations we still need to do.
    fn iterations_number(&self) -> usize;
    /// Return if nothing is left to do.
    fn completed(&self) -> bool {
        self.iterations_number() == 0
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        Box::new(std::iter::successors(Some(1), |i| Some(2 * i)))
    }
    /// Reduce on one block.
    fn block_reduce<ID, OP>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        schedule_reduce(self, &identity, &op, identity())
    }
}
