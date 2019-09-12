//! Adaptor to ensure all final tasks are at an even level.
//! This is especially nice for the merge sort : you are sure all data is where it should be.

// This file also serves as an example illustrating how to implement simple adaptors.
use crate::prelude::*;

// step one : define your structure

/// Iterator where all tasks are guaranteed at an even level from the root.
pub struct EvenLevels<I> {
    pub(crate) even: bool,
    pub(crate) iterator: I,
}

// step two : implement Divisible

impl<I: Divisible> Divisible for EvenLevels<I> {
    fn is_divisible(&self) -> bool {
        // even if we are not divisible, if we are not on an even level, divide once more
        self.iterator.is_divisible() || !self.even
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            EvenLevels {
                even: !self.even,
                iterator: left,
            },
            EvenLevels {
                even: !self.even,
                iterator: right,
            },
        )
    }
}

// step three, before implementing ParallelIterator we start by choosing the types
// and registering them in the auxilliary traits.
// we implement "ItemProducer" and "FinitePart".

impl<I: ParallelIterator> ItemProducer for EvenLevels<I> {
    type Owner = EvenLevels<I::Owner>;
    type Item = I::Item;
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> Borrowed<'e> for EvenLevels<I> {
    type ParIter = EvenLevels<<I::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter = <I::Owner as Borrowed<'e>>::SeqIter;
}

// step four, let's implement ParallelIterator

impl<I: ParallelIterator> ParallelIterator for EvenLevels<I> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        EvenLevels {
            even: self.even,
            iterator: self.iterator.borrow_on_left_for(size),
        }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size)
    }
}

// last step, let's implement FiniteParallelIterator

impl<I: FiniteParallelIterator> FiniteParallelIterator for EvenLevels<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
