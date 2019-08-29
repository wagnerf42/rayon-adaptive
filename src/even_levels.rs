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
    type Item = I::Item;
}

impl<'extraction, I: ParallelIterator> FinitePart<'extraction> for EvenLevels<I> {
    type ParIter = EvenLevels<<I as FinitePart<'extraction>>::ParIter>;
    type SeqIter = <I as FinitePart<'extraction>>::SeqIter;
}

// step four, let's implement ParallelIterator

impl<I: ParallelIterator> ParallelIterator for EvenLevels<I> {
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter {
        EvenLevels {
            even: self.even,
            iterator: self.iterator.borrow_on_left_for(size),
        }
    }

    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size)
    }
}

// last step, let's implement FiniteParallelIterator

impl<I: FiniteParallelIterator> FiniteParallelIterator for EvenLevels<I> {
    type Iter = I::Iter;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::Iter {
        self.iterator.to_sequential()
    }
}
