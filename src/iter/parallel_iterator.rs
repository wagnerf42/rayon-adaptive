//! `ParallelIterator` structure. We use a struct here and not a trait like in rayon.
//! This way we can have an easier code specialisation by having different structs for different
//! types of iterators.
use super::BaseIterator;
use crate::prelude::*;
use std::iter::{once, Once};

/// ParallelIterator is a struct here, not a trait.
/// Doing that enables us an easy specialisation code.
pub struct ParallelIterator<Input: Divisible + Edible>(pub Input); // TODO: put content private again

impl<Input: Divisible + Edible> Divisible for ParallelIterator<Input> {
    fn base_length(&self) -> usize {
        self.0.base_length()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.0.divide();
        (ParallelIterator(left), ParallelIterator(right))
    }
}

impl<Input: Divisible + Edible> Edible for ParallelIterator<Input> {
    type Item = <Input::SequentialIterator as Iterator>::Item;
    type SequentialIterator = Input::SequentialIterator;
    fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
        let (remaining_input, iterator) = self.0.iter(size);
        (ParallelIterator(remaining_input), iterator)
    }
}

impl<Input: Divisible + Edible> BaseIterator for ParallelIterator<Input> {
    type BlocksIterator = Once<Self>;
    fn blocks(self) -> Self::BlocksIterator {
        once(self)
    }
}
