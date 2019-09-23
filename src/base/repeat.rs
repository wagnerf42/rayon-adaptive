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
}

impl<T: Clone + Send + Sync> Powered for Repeat<T> {
    type Power = Indexed;
}

impl<'a, T: Clone + Send + Sync> ItemProducer for BorrowedRepeat<'a, T> {
    type Item = T;
}

impl<'e, T: Clone + Send + Sync> ParBorrowed<'e> for Repeat<T> {
    type Iter = BorrowedRepeat<'e, T>;
}

impl<'a, 'e, T: Clone + Send + Sync> SeqBorrowed<'e> for BorrowedRepeat<'a, T> {
    type Iter = std::iter::Take<std::iter::Repeat<T>>;
}

impl<'e, T: Clone + Send + Sync> Divisible for BorrowedRepeat<'e, T> {
    fn should_be_divided(&self) -> bool {
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
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowedRepeat {
            element: Dislocated::new(&self.element),
            size,
        }
    }
}

impl<'a, T: Clone + Send + Sync> BorrowingParallelIterator for BorrowedRepeat<'a, T> {
    fn iterations_number(&self) -> usize {
        self.size
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.size -= size;
        std::iter::repeat(std::ops::Deref::deref(&self.element).clone()).take(size)
    }
}

pub fn repeat<T: Clone + Send>(elt: T) -> Repeat<T> {
    Repeat { element: elt }
}
