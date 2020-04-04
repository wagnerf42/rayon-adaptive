use crate::prelude::*;

///This special zip will be biased towards A.
///Hence all division will be decided by A, and B will simply try to mimic what A does.
///This is useful when you don't know how A will divide itself before the division actually happens
pub struct DirectionalZip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> ItemProducer for DirectionalZip<A, B>
where
    A: ItemProducer,
    B: ItemProducer,
{
    type Item = (A::Item, B::Item);
}

impl<A, B> Powered for DirectionalZip<A, B> {
    type Power = Indexed;
}

impl<'e, A, B> ParBorrowed<'e> for DirectionalZip<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator,
{
    type Iter = DirectionalZip<<A as ParBorrowed<'e>>::Iter, <B as ParBorrowed<'e>>::Iter>;
}

impl<'e, A, B> SeqBorrowed<'e> for DirectionalZip<A, B>
where
    A: BorrowingParallelIterator,
    B: BorrowingParallelIterator,
{
    type Iter = std::iter::Zip<<A as SeqBorrowed<'e>>::Iter, <B as SeqBorrowed<'e>>::Iter>;
}

impl<A, B> ParallelIterator for DirectionalZip<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        //Final verdict should be given by a
        self.a
            .bound_iterations_number(self.b.bound_iterations_number(size))
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        DirectionalZip {
            a: self.a.par_borrow(size),
            b: self.b.par_borrow(size),
        }
    }
}

impl<A, B> BorrowingParallelIterator for DirectionalZip<A, B>
where
    A: BorrowingParallelIterator,
    B: BorrowingParallelIterator,
{
    fn iterations_number(&self) -> usize {
        let iterations_a = self.a.iterations_number();
        let iterations_b = self.b.iterations_number();
        debug_assert!(iterations_a == iterations_b);
        iterations_a
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.a.seq_borrow(size).zip(self.b.seq_borrow(size))
    }
    fn part_completed(&self) -> bool {
        self.a.part_completed()
    }
}

impl<A, B> Divisible for DirectionalZip<A, B>
where
    A: Divisible + BorrowingParallelIterator,
    B: Divisible,
{
    fn should_be_divided(&self) -> bool {
        //Not entirely sure, maybe should "and" with b
        self.a.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left_a, right_a) = self.a.divide();
        let (left_b, right_b) = self.b.divide_at(left_a.iterations_number());
        (
            DirectionalZip {
                a: left_a,
                b: left_b,
            },
            DirectionalZip {
                a: right_a,
                b: right_b,
            },
        )
    }
}
