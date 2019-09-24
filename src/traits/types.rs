use crate::prelude::*;

pub struct Standard;
pub struct Indexed;

pub trait Powered {
    type Power;
}

pub trait MinPower<B> {
    type Min;
}

impl<B> MinPower<B> for Standard {
    type Min = Standard;
}

impl<B> MinPower<B> for Indexed {
    type Min = B;
}

pub trait ItemProducer {
    type Item: Send + Sized;
}

pub trait ParBorrowed<'e>: ItemProducer {
    type Iter: BorrowingParallelIterator<Item = Self::Item>;
}

pub trait SeqBorrowed<'e>: ItemProducer {
    type Iter: Iterator<Item = Self::Item>;
}
