//! Partition adaptor
use crate::prelude::*;
use crate::BasicPower;
use derive_divisible::ParallelIterator;

/// Partition iterator adapter, dividing the underlying iterator into partitions of
/// size N.
#[derive(ParallelIterator)]
#[power(BasicPower)]
#[item(I::Item)]
#[sequential_iterator(I::SequentialIterator)]
#[iterator_extraction(i)]
#[trait_bounds(I: ParallelIterator)]
pub struct Partition<I> {
    pub(crate) iterator: I,
    #[divide_by(clone)]
    pub(crate) task_size: usize,
    #[divide_by(clone)]
    pub(crate) degree: usize,
}

impl<I: ParallelIterator> Divisible for Partition<I> {
    type Power = BasicPower;

    fn base_length(&self) -> Option<usize> {
        self.iterator.base_length()
    }

    fn divide_at(mut self, _index: usize) -> (Self, Self) {
        let base_length = self.iterator.base_length().unwrap();
        let divide = if base_length <= self.task_size {
            base_length
        } else {
            base_length - self.task_size
        };

        let (left, right) = self.iterator.divide_at(divide);
        self.iterator = left;

        let task_size = right.base_length().unwrap() / self.degree + 1;
        let degree = self.degree;
        (
            self,
            Partition {
                iterator: right,
                task_size,
                degree,
            },
        )
    }
}
