use crate::prelude::*;

pub struct NotIndexed();
pub struct Indexed();

pub trait ItemProducer: Sized {
    type Owner: for<'e> Borrowed<'e>
        + ItemProducer<Item = Self::Item, Owner = Self::Owner, Power = Self::Power>
        + ParallelIterator;
    type Item: Send + Sized;
    type Power;
}

pub trait Borrowed<'e>: ItemProducer {
    type ParIter: FiniteParallelIterator
        + Divisible
        + ItemProducer<Item = Self::Item, Owner = Self::Owner, Power = Self::Power>;
    type SeqIter: Iterator<Item = Self::Item>;
}

