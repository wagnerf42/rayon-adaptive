use crate::prelude::*;

pub struct Cloned<I> {
    pub(crate) iterator: I,
}

impl<I: Divisible> Divisible for Cloned<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (Cloned { iterator: left }, Cloned { iterator: right })
    }
}

impl<'a, T, I> ItemProducer for Cloned<I>
where
    I: ParallelIterator<Item = &'a T>,
    T: 'a + Clone + Send + Sync,
{
    type Owner = Cloned<I::Owner>;
    type Item = T;
}

impl<'e, 'a, T, I> Borrowed<'e> for Cloned<I>
where
    I: ParallelIterator<Item = &'a T>,
    T: 'a + Clone + Send + Sync,
{
    type ParIter = Cloned<<I::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter = std::iter::Cloned<<I::Owner as Borrowed<'e>>::SeqIter>;
}

impl<'a, T, I> ParallelIterator for Cloned<I>
where
    I: ParallelIterator<Item = &'a T>,
    T: 'a + Clone + Send + Sync,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        Cloned {
            iterator: self.iterator.borrow_on_left_for(size),
        }
    }

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.iterator.sequential_borrow_on_left_for(size).cloned()
    }
}

impl<'a, T, I> FiniteParallelIterator for Cloned<I>
where
    I: FiniteParallelIterator<Item = &'a T>,
    T: 'a + Clone + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
