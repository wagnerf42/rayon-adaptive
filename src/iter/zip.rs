use crate::prelude::*;

pub struct Zip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> ItemProducer for Zip<A, B>
where
    A: ItemProducer,
    B: ItemProducer,
{
    type Item = (A::Item, B::Item);
}

impl<A, B> Powered for Zip<A, B> {
    type Power = Indexed;
}

impl<'e, A, B> ParBorrowed<'e> for Zip<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator,
{
    type Iter = Zip<<A as ParBorrowed<'e>>::Iter, <B as ParBorrowed<'e>>::Iter>;
}

impl<'e, A, B> SeqBorrowed<'e> for Zip<A, B>
where
    A: BorrowingParallelIterator,
    B: BorrowingParallelIterator,
{
    type Iter = std::iter::Zip<<A as SeqBorrowed<'e>>::Iter, <B as SeqBorrowed<'e>>::Iter>;
}

impl<A, B> ParallelIterator for Zip<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.b
            .bound_iterations_number(self.a.bound_iterations_number(size))
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        Zip {
            a: self.a.par_borrow(size),
            b: self.b.par_borrow(size),
        }
    }
}

impl<A, B> BorrowingParallelIterator for Zip<A, B>
where
    A: BorrowingParallelIterator,
    B: BorrowingParallelIterator,
{
    fn iterations_number(&self) -> usize {
        let iterations_a = self.a.iterations_number();
        let iterations_b = self.b.iterations_number();
        assert_eq!(iterations_a, iterations_b);
        iterations_a
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.a.seq_borrow(size).zip(self.b.seq_borrow(size))
    }
    fn part_completed(&self) -> bool {
        //Division has to be unanimous
        self.a.part_completed() || self.b.part_completed()
    }
}

impl<A, B> Divisible for Zip<A, B>
where
    A: Divisible,
    B: Divisible,
{
    fn should_be_divided(&self) -> bool {
        self.a.should_be_divided() || self.b.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left_a, right_a) = self.a.divide();
        let (left_b, right_b) = self.b.divide();
        (
            Zip {
                a: left_a,
                b: left_b,
            },
            Zip {
                a: right_a,
                b: right_b,
            },
        )
    }
}
