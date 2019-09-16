use crate::prelude::*;

/// Parallel iterator which may be of two different types.
/// This is simplifying the code in other places like flatmap.
enum EitherIter<I, J> {
    I(I),
    J(J),
}

impl<I, J> ItemProducer for EitherIter<I, J>
where
    I: ParallelIterator,
    J: ParallelIterator<Item = I::Item, Power = I::Power>, // TODO: put the min between the two powers here
{
    type Item = I::Item;
    type Owner = EitherIter<I::Owner, J::Owner>;
    type Power = I::Power;
}

impl<'e, I, J> Borrowed<'e> for EitherIter<I, J>
where
    I: ParallelIterator,
    J: ParallelIterator<Item = I::Item, Power = I::Power>,
{
    type ParIter =
        EitherIter<<I::Owner as Borrowed<'e>>::ParIter, <J::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter =
        EitherSeqIter<<I::Owner as Borrowed<'e>>::SeqIter, <J::Owner as Borrowed<'e>>::SeqIter>;
}

impl<I, J> Divisible for EitherIter<I, J>
where
    I: Divisible,
    J: Divisible,
{
    fn is_divisible(&self) -> bool {
        match self {
            EitherIter::I(i) => i.is_divisible(),
            EitherIter::J(j) => j.is_divisible(),
        }
    }
    fn divide(self) -> (Self, Self) {
        match self {
            EitherIter::I(i) => {
                let (left, right) = i.divide();
                (EitherIter::I(left), EitherIter::I(right))
            }
            EitherIter::J(j) => {
                let (left, right) = j.divide();
                (EitherIter::J(left), EitherIter::J(right))
            }
        }
    }
}

enum EitherSeqIter<I, J> {
    I(I),
    J(J),
}

impl<I, J> ParallelIterator for EitherIter<I, J>
where
    I: ParallelIterator,
    J: ParallelIterator<Item = I::Item, Power = I::Power>,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        unimplemented!()
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        unimplemented!()
    }
}

impl<I, J> FiniteParallelIterator for EitherIter<I, J>
where
    I: FiniteParallelIterator,
    J: FiniteParallelIterator<Item = I::Item, Power = I::Power>,
{
    fn len(&self) -> usize {
        match self {
            EitherIter::I(i) => i.len(),
            EitherIter::J(j) => j.len(),
        }
    }
}

impl<I, J> Iterator for EitherSeqIter<I, J>
where
    I: Iterator,
    J: Iterator<Item = I::Item>,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherSeqIter::I(i) => i.next(),
            EitherSeqIter::J(j) => j.next(),
        }
    }
}
