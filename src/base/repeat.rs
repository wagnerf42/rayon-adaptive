use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct Repeat<T> {
    element: T,
}

pub struct BorrowedRepeat<'a, T: Sync> {
    element: Dislocated<'a, T>,
    size: usize,
}

impl<T: Clone + Send + Sync> ItemProducer for Repeat<T> {
    type Item = T;
    type Owner = Self;
    type Power = Indexed;
}

impl<'e, T: Clone + Send + Sync> ItemProducer for BorrowedRepeat<'e, T> {
    type Item = T;
    type Owner = Repeat<T>;
    type Power = Indexed;
}

impl<'e, T: Clone + Send + Sync> Borrowed<'e> for Repeat<T> {
    type ParIter = BorrowedRepeat<'e, T>;
    type SeqIter = std::iter::Take<std::iter::Repeat<T>>;
}

impl<'e, T: Clone + Send + Sync> Divisible for BorrowedRepeat<'e, T> {
    fn is_divisible(&self) -> bool {
        self.size > 1
    }
    fn divide(self) -> (Self, Self) {
        let left_size = self.size / 2;
        let right_size = self.size - left_size;
        (
            BorrowedRepeat {
                element: self.element.clone(),
                size: left_size,
            },
            BorrowedRepeat {
                element: self.element,
                size: right_size,
            },
        )
    }
}

impl<T: Clone + Send + Sync> ParallelIterator for Repeat<T> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowedRepeat {
            element: Dislocated::new(&self.element),
            size,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        std::iter::repeat(self.element.clone()).take(size)
    }
}

impl<'a, T: Clone + Send + Sync> ParallelIterator for BorrowedRepeat<'a, T> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        self.size -= size;
        BorrowedRepeat {
            element: self.element.clone(),
            size,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        self.size -= size;
        let r: &T = &self.element;
        //TODO: we have a slight overhead when cloning here.
        //we could avoid it by implementing our own sequential repeat
        //but I don't think it's worth it.
        std::iter::repeat(r.clone()).take(size)
    }
}

impl<'a, T: Clone + Send + Sync> FiniteParallelIterator for BorrowedRepeat<'a, T> {
    fn len(&self) -> usize {
        self.size
    }
}

pub fn repeat<T: Clone + Send>(elt: T) -> Repeat<T> {
    Repeat { element: elt }
}
