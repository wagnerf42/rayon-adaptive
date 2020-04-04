use crate::prelude::*;

pub struct DampenLocalDivision<I> {
    pub(crate) iterator: I,
    pub(crate) counter: usize,
    pub(crate) created_by: Option<usize>,
}

impl<I: ItemProducer> ItemProducer for DampenLocalDivision<I> {
    type Item = I::Item;
}

impl<I: Powered> Powered for DampenLocalDivision<I> {
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> ParBorrowed<'e> for DampenLocalDivision<I> {
    type Iter = DampenLocalDivision<<I as ParBorrowed<'e>>::Iter>;
}

impl<'e, I: BorrowingParallelIterator> SeqBorrowed<'e> for DampenLocalDivision<I> {
    type Iter = <I as SeqBorrowed<'e>>::Iter;
}

impl<I: ParallelIterator> ParallelIterator for DampenLocalDivision<I> {
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.iterator.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        DampenLocalDivision {
            iterator: self.iterator.par_borrow(size),
            counter: self.counter,
            created_by: self.created_by,
        }
    }
}

impl<I: BorrowingParallelIterator> BorrowingParallelIterator for DampenLocalDivision<I> {
    fn iterations_number(&self) -> usize {
        self.iterator.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.iterator.seq_borrow(size)
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        self.iterator.micro_blocks_sizes()
    }
    fn part_completed(&self) -> bool {
        self.iterator.part_completed()
    }
}

impl<I: BorrowingParallelIterator> Divisible for DampenLocalDivision<I> {
    fn should_be_divided(&self) -> bool {
        self.iterator.should_be_divided() && self.counter != 0
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
