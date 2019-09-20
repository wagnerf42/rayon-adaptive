use crate::prelude::*;

pub trait ItemProducer {
    type Item: Send + Sized;
}

pub trait ParBorrowed<'e>: ItemProducer {
    type Iter: BorrowingParallelIterator<Item = Self::Item>;
}

pub trait SeqBorrowed<'e>: ItemProducer {
    type Iter: Iterator<Item = Self::Item>;
}
