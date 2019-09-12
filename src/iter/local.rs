use crate::prelude::*;

pub struct DampenLocalDivision<I> {
    pub(crate) iterator: I,
    pub(crate) counter: usize,
    pub(crate) created_by: Option<usize>,
}

impl<I: FiniteParallelIterator + Divisible> Divisible for DampenLocalDivision<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible() && self.counter != 0
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        let current_thread = rayon::current_thread_index();
        let new_counter = if current_thread == self.created_by {
            if self.counter == 0 {
                // we should not assume counter is not 0
                0
            } else {
                self.counter - 1
            }
        } else {
            (rayon::current_num_threads() as f64).log(2.0).ceil() as usize
        };
        (
            DampenLocalDivision {
                iterator: left,
                counter: new_counter,
                created_by: current_thread,
            },
            DampenLocalDivision {
                iterator: right,
                counter: new_counter,
                created_by: current_thread,
            },
        )
    }
}

impl<I: ParallelIterator> ItemProducer for DampenLocalDivision<I> {
    type Owner = DampenLocalDivision<I::Owner>;
    type Item = I::Item;
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> Borrowed<'e> for DampenLocalDivision<I> {
    type ParIter = DampenLocalDivision<<I::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter = <I::Owner as Borrowed<'e>>::SeqIter;
}

impl<I: ParallelIterator> ParallelIterator for DampenLocalDivision<I> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        DampenLocalDivision {
            iterator: self.iterator.borrow_on_left_for(size),
            counter: self.counter,
            created_by: self.created_by,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size)
    }
}

impl<I: FiniteParallelIterator> FiniteParallelIterator for DampenLocalDivision<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
