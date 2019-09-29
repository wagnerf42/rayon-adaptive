use crate::prelude::*;

pub struct Take<I> {
    pub(crate) iterator: I,
    pub(crate) n: usize,
}

impl<I: ItemProducer> ItemProducer for Take<I> {
    type Item = I::Item;
}

impl<I: Powered> Powered for Take<I> {
    type Power = Indexed;
}

impl<'l, I: ParBorrowed<'l>> ParBorrowed<'l> for Take<I> {
    type Iter = I::Iter;
}

impl<I: ParallelIterator> ParallelIterator for Take<I> {
    fn bound_iterations_number(&self, input_size: usize) -> usize {
        std::cmp::min(self.n, input_size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        let old_size = self.n;
        self.n = std::cmp::max(0, self.n - size);
        self.iterator.par_borrow(std::cmp::min(old_size, size))
    }
}
