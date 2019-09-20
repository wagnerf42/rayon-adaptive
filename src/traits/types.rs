use crate::prelude::*;

pub trait Borrowed<'e>: Owner {
    type Iter: FiniteParallelIterator + Divisible;
}

pub trait ParallelIterator: Sized
where
    Self: for<'e> Borrowed<'e>,
{
    type Item: Send + Sized;
    type Power;
    type Finiteness;
}

pub trait Borrower: Sized {}
