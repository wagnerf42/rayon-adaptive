use crate::scheduler::schedule_reduce;
mod divisible;
// mod finite_parallel_iterator;
mod indexed;
mod into_iterator;
mod into_parallel_ref;
// mod parallel_iterator;
// mod types;
//
pub use divisible::Divisible;
// pub use finite_parallel_iterator::FiniteParallelIterator;
pub use indexed::IndexedParallelIterator;
pub use into_iterator::IntoParallelIterator;
pub use into_parallel_ref::IntoParallelRefIterator;
// pub use parallel_iterator::ParallelIterator;
// pub use types::{Borrowed, Indexed, ItemProducer, NotIndexed};

pub struct True;
pub struct False;

pub trait ItemProducer {
    type Item: Send + Sized;
}

pub trait ParBorrowed<'e>: ItemProducer {
    type Iter: BorrowingParallelIterator<Item = Self::Item>;
}

pub trait ParallelIterator
where
    Self: for<'e> ParBorrowed<'e>,
{
    type IsFinite;
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter;
    fn bound_size(&self, size: usize) -> usize {
        size
    }
}

pub trait SeqBorrowed<'e>: ItemProducer {
    type Iter: Iterator<Item = Self::Item>;
}

pub trait BorrowingParallelIterator: Divisible + ItemProducer + Send
where
    Self: for<'e> SeqBorrowed<'e>,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter;
    fn len(&self) -> usize;
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
