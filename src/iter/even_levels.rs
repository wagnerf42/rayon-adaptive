//! Adaptor to ensure all final tasks are at an even level.
//! This is especially nice for the merge sort : you are sure all data is where it should be.

// This file also serves as an example illustrating how to implement simple adaptors.
use crate::prelude::*;

// step one : define your structure

/// Iterator where all tasks are guaranteed at an even level from the root.
pub struct EvenLevels<I> {
    pub(crate) even: bool,
    pub(crate) base: I,
}

// step two : set all associated types.

// let's choose our Items
impl<I: ItemProducer> ItemProducer for EvenLevels<I> {
    type Item = I::Item;
}

// let's choose if we are indexed or not
impl<I: Powered> Powered for EvenLevels<I> {
    type Power = I::Power;
}

// let's choose what we get once borrowed for parallel bases
impl<'e, I: ParallelIterator> ParBorrowed<'e> for EvenLevels<I> {
    type Iter = EvenLevels<<I as ParBorrowed<'e>>::Iter>;
}

// let's choose what we get once borrowed for sequential bases
impl<'e, I: BorrowingParallelIterator> SeqBorrowed<'e> for EvenLevels<I> {
    type Iter = <I as SeqBorrowed<'e>>::Iter;
}

// third step : implement borrowing traits.

impl<I> ParallelIterator for EvenLevels<I>
where
    I: ParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        EvenLevels {
            even: true,
            base: self.base.par_borrow(size),
        }
    }
}

impl<I> BorrowingParallelIterator for EvenLevels<I>
where
    I: BorrowingParallelIterator,
{
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.base.seq_borrow(size)
    }
}

// last step : implement Divisible

impl<I: Divisible> Divisible for EvenLevels<I> {
    fn should_be_divided(&self) -> bool {
        // even if base should not be divided, if we are not on an even level, divide once more
        self.base.should_be_divided() || !self.even
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            EvenLevels {
                even: !self.even,
                base: left,
            },
            EvenLevels {
                even: !self.even,
                base: right,
            },
        )
    }
}
