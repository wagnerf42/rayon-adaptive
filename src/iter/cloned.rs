use crate::prelude::*;

pub struct Cloned<I> {
    pub(crate) base: I,
}

impl<'a, T, I> ItemProducer for Cloned<I>
where
    T: Clone + Send + Sync + 'a,
    I: ItemProducer<Item = &'a T>,
{
    type Item = T;
}

impl<I: MaybeIndexed> MaybeIndexed for Cloned<I> {
    type IsIndexed = I::IsIndexed;
}

impl<'e, 'a, I, T> ParBorrowed<'e> for Cloned<I>
where
    T: Clone + Send + Sync + 'a,
    I: ParallelIterator<Item = &'a T>,
{
    type Iter = Cloned<<I as ParBorrowed<'e>>::Iter>;
}

impl<'e, 'a, T, I> SeqBorrowed<'e> for Cloned<I>
where
    T: Clone + Send + Sync + 'a,
    I: BorrowingParallelIterator<Item = &'a T>,
{
    type Iter = std::iter::Cloned<<I as SeqBorrowed<'e>>::Iter>;
}

impl<I: Divisible> Divisible for Cloned<I> {
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (Cloned { base: left }, Cloned { base: right })
    }
}

impl<'a, T, I> BorrowingParallelIterator for Cloned<I>
where
    T: Clone + Send + Sync + 'a,
    I: BorrowingParallelIterator<Item = &'a T>,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.base.seq_borrow(size).cloned()
    }
    fn len(&self) -> usize {
        self.base.len()
    }
}

impl<'a, T, I> ParallelIterator for Cloned<I>
where
    T: Clone + Send + Sync + 'a,
    I: ParallelIterator<Item = &'a T>,
{
    type IsFinite = I::IsFinite;
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        Cloned {
            base: self.base.par_borrow(size),
        }
    }
}
