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

pub trait ParallelIterator
where
    Self: for<'e> ParBorrowed<'e>,
{
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter;
    /// Takes the number of iterations requested by the user
    /// and return the number we can really process.
    fn bound_iterations_number(&self, size: usize) -> usize {
        size
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
    fn reduce<ID, OP>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        schedule_reduce(self, &identity, &op, identity())
    }
}
