use crate::prelude::*;

pub struct DampenLocalDivision<I> {
    pub(crate) iterator: I,
    pub(crate) counter: usize,
    pub(crate) created_by: Option<usize>,
}

impl<I: ParallelIterator> Divisible for DampenLocalDivision<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible() && self.counter != 0
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        let current_thread = rayon::current_thread_index();
        let new_counter = if current_thread == self.created_by {
            self.counter - 1
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

impl<I: FiniteParallelIterator> FiniteParallelIterator for DampenLocalDivision<I> {
    type SequentialIterator = I::SequentialIterator;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential()
    }
}

impl<I: ParallelIterator> ParallelIterator for DampenLocalDivision<I> {
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::Iter {
        DampenLocalDivision {
            iterator: self.iterator.borrow_on_left_for(size),
            counter: self.counter,
            created_by: self.created_by,
        }
    }
}

impl<'extraction, I: ParallelIterator> FinitePart<'extraction> for DampenLocalDivision<I> {
    type Iter = DampenLocalDivision<<I as FinitePart<'extraction>>::Iter>;
}

impl<I: ParallelIterator> ItemProducer for DampenLocalDivision<I> {
    type Item = <I as ItemProducer>::Item;
}
