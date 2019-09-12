use crate::prelude::*;

pub struct Take<I> {
    pub(crate) iterator: I,
    pub(crate) n: usize,
}

impl<I: ParallelIterator> ItemProducer for Take<I> {
    type Owner = I::Owner;
    type Item = I::Item;
}

impl<'e, I: ParallelIterator> Borrowed<'e> for Take<I> {
    type ParIter = <I::Owner as Borrowed<'e>>::ParIter;
    type SeqIter = <I::Owner as Borrowed<'e>>::SeqIter;
}

impl<I: ParallelIterator> ParallelIterator for Take<I> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        let real_size = std::cmp::min(size, self.n);
        self.n -= real_size;
        self.iterator.borrow_on_left_for(real_size)
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        let real_size = std::cmp::min(size, self.n);
        self.n -= real_size;
        self.iterator.sequential_borrow_on_left_for(real_size)
    }
}

impl<I: FiniteParallelIterator> FiniteParallelIterator for Take<I> {
    fn len(&self) -> usize {
        std::cmp::min(self.n, self.iterator.len())
    }
}

impl<I: ParallelIterator> IndexedParallelIterator for Take<I> {}
