use crate::prelude::*;

pub struct Zip<A, B> {
    pub(crate) a: A,
    pub(crate) b: B,
}

// we want to constrain that only borrowed iterators are divisible
// to ensure that we have an equal size.
// this way we don't need a divide_at_index.
pub struct BorrowingZip<A, B> {
    a: A,
    b: B,
}

impl<A: Divisible, B: Divisible> Divisible for BorrowingZip<A, B> {
    fn is_divisible(&self) -> bool {
        self.a.is_divisible() && self.b.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left_a, right_a) = self.a.divide();
        let (left_b, right_b) = self.b.divide();
        (
            BorrowingZip {
                a: left_a,
                b: left_b,
            },
            BorrowingZip {
                a: right_a,
                b: right_b,
            },
        )
    }
}

impl<A, B> ItemProducer for Zip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    type Owner = Zip<A::Owner, B::Owner>;
    type Item = (A::Item, B::Item);
    type Power = Indexed;
}

impl<A, B> ItemProducer for BorrowingZip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    type Owner = Zip<A::Owner, B::Owner>;
    type Item = (A::Item, B::Item);
    type Power = Indexed;
}

impl<'e, A, B> Borrowed<'e> for Zip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    type ParIter =
        BorrowingZip<<A::Owner as Borrowed<'e>>::ParIter, <B::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter =
        std::iter::Zip<<A::Owner as Borrowed<'e>>::SeqIter, <B::Owner as Borrowed<'e>>::SeqIter>;
}

impl<'e, A, B> Borrowed<'e> for BorrowingZip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    type ParIter =
        BorrowingZip<<A::Owner as Borrowed<'e>>::ParIter, <B::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter =
        std::iter::Zip<<A::Owner as Borrowed<'e>>::SeqIter, <B::Owner as Borrowed<'e>>::SeqIter>;
}

impl<A, B> ParallelIterator for Zip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    fn bound_size(&self, size: usize) -> usize {
        self.b.bound_size(self.a.bound_size(size))
    }
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        let borrowed_a = self.a.borrow_on_left_for(size);
        let borrowed_b = self.b.borrow_on_left_for(size);
        BorrowingZip {
            a: borrowed_a,
            b: borrowed_b,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        // the sequential zip will take care of unequal sizes
        self.a
            .sequential_borrow_on_left_for(size)
            .zip(self.b.sequential_borrow_on_left_for(size))
    }
}

impl<A, B> ParallelIterator for BorrowingZip<A, B>
where
    A: IndexedParallelIterator,
    B: IndexedParallelIterator,
{
    fn bound_size(&self, size: usize) -> usize {
        self.b.bound_size(self.a.bound_size(size))
    }
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowingZip {
            a: self.a.borrow_on_left_for(size),
            b: self.b.borrow_on_left_for(size),
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.a
            .sequential_borrow_on_left_for(size)
            .zip(self.b.sequential_borrow_on_left_for(size))
    }
}

impl<A, B> FiniteParallelIterator for BorrowingZip<A, B>
where
    A: IndexedParallelIterator + FiniteParallelIterator,
    B: IndexedParallelIterator + FiniteParallelIterator,
{
    fn len(&self) -> usize {
        std::cmp::min(self.a.len(), self.b.len())
    }
}

impl<A, B> FiniteParallelIterator for Zip<A, B>
where
    A: IndexedParallelIterator + FiniteParallelIterator,
    B: IndexedParallelIterator + FiniteParallelIterator,
{
    fn len(&self) -> usize {
        std::cmp::min(self.a.len(), self.b.len())
    }
}
