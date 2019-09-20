mod divisible;
// mod finite_parallel_iterator;
// mod indexed;
// mod into_iterator;
// mod into_parallel_ref;
// mod parallel_iterator;
// mod types;
//
pub use divisible::Divisible;
// pub use finite_parallel_iterator::FiniteParallelIterator;
// pub use indexed::IndexedParallelIterator;
// pub use into_iterator::IntoParallelIterator;
// pub use into_parallel_ref::IntoParallelRefIterator;
// pub use parallel_iterator::ParallelIterator;
// pub use types::{Borrowed, Indexed, ItemProducer, NotIndexed};

pub struct True;
pub struct False;

pub trait ItemProducer {
    type Item: Send + Sized;
}

pub trait MaybeIndexed {
    type IsIndexed;
}

pub trait ParBorrowed<'e>: ItemProducer + MaybeIndexed {
    type Iter: BorrowingParallelIterator<Item = Self::Item, IsIndexed = Self::IsIndexed>;
}

pub trait ParallelIterator
where
    Self: for<'e> ParBorrowed<'e>,
{
    type IsFinite;
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter;
}

pub trait SeqBorrowed<'e>: ItemProducer {
    type Iter: Iterator<Item = Self::Item>;
}

pub trait BorrowingParallelIterator: Divisible + MaybeIndexed + ItemProducer
where
    Self: for<'e> SeqBorrowed<'e>,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter;
}
