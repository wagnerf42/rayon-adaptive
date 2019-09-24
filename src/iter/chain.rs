use crate::base::either_iter::{EitherIter, EitherSeqIter};
use crate::prelude::*;

pub struct Chain<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

impl<A, B> ItemProducer for Chain<A, B>
where
    A: ItemProducer,
    B: ItemProducer<Item = A::Item>,
{
    type Item = A::Item;
}

impl<A, B> Powered for Chain<A, B>
where
    A: Powered,
    B: Powered,
    B::Power: MinPower<A::Power>,
{
    type Power = <B::Power as MinPower<A::Power>>::Min;
}

impl<'e, A, B> ParBorrowed<'e> for Chain<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator<Item = A::Item>,
    B::Power: MinPower<A::Power>,
{
    type Iter = EitherIter<<A as ParBorrowed<'e>>::Iter, <B as ParBorrowed<'e>>::Iter>;
}

impl<A, B> ParallelIterator for Chain<A, B>
where
    A: ParallelIterator,
    B: ParallelIterator<Item = A::Item>,
    B::Power: MinPower<A::Power>,
{
    // we lie here to ensure the macro block separates
    // A and B.
    // this means that the scheduler must always loop on blocks until empty.
    fn bound_iterations_number(&self, size: usize) -> usize {
        let a_size = self.a.bound_iterations_number(size);
        if a_size != 0 {
            a_size
        } else {
            self.b.bound_iterations_number(size)
        }
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        if !self.a.completed() {
            EitherIter::I(self.a.par_borrow(size))
        } else {
            EitherIter::J(self.b.par_borrow(size))
        }
    }
}
