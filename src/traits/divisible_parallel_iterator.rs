use crate::prelude::{
    BorrowingParallelIterator, Divisible, ParBorrowed, ParallelIterator, Powered, SeqBorrowed,
};
use crate::traits::Indexed;
use std::iter::{once, Once};
/// This trait provides a shortcut to making a parallel iterator. If this trait is implemented, the
/// type shall also automatically implement ParallelIterator and IndexedParallelIterator traits.
/// The catch is that even while making macro blocks, this divide_at_index() method will be called.
/// If the divide_at_index() method is expensive, go the longer route and implement
/// ParallelIterator yourself.
pub trait DivisibleParallelIterator: Send + Sized {
    fn base_length(&self) -> usize;
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self;
    /// Will wrap self in a type that is also a divisible parallel iterator. Should allow quick
    /// nesting of parallel iterators.
    fn wrap_iter(self) -> Wrapper<Self> {
        Wrapper { inner_iter: self }
    }
}

pub struct Wrapper<T> {
    inner_iter: T,
}

impl<T: DivisibleParallelIterator> DivisibleParallelIterator for Wrapper<T> {
    fn base_length(&self) -> usize {
        self.inner_iter.base_length() //ASK: doesn't feel right.
    }
    fn cut_at_index(&mut self, index: usize) -> Self {
        let left_side = self.inner_iter.cut_at_index(index);
        Wrapper {
            inner_iter: left_side,
        }
    }
}

impl<T> IntoIterator for Wrapper<T> {
    type Item = T;
    type IntoIter = Once<T>;
    fn into_iter(self) -> Self::IntoIter {
        once(self.inner_iter)
    }
}

impl<I: DivisibleParallelIterator> Powered for I {
    type Power = Indexed;
}

impl<'l, I: DivisibleParallelIterator + IntoIterator> SeqBorrowed<'l> for I
where
    I::Item: Sized + Send,
{
    type Iter = I::IntoIter;
}

impl<I: DivisibleParallelIterator> Divisible for I {
    fn should_be_divided(&self) -> bool {
        self.base_length() > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mylen = self.base_length();
        (self.cut_at_index(mylen / 2), self)
    }
}

impl<'l, I: DivisibleParallelIterator + IntoIterator> ParBorrowed<'l> for I
where
    I::Item: Sized + Send,
{
    type Iter = I;
}

impl<I: DivisibleParallelIterator + IntoIterator> BorrowingParallelIterator for I
where
    I::Item: Sized + Send,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.cut_at_index(size).into_iter()
    }
    fn iterations_number(&self) -> usize {
        self.base_length()
    }
}

impl<I: DivisibleParallelIterator + IntoIterator> ParallelIterator for I
where
    I::Item: Sized + Send,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        std::cmp::min(size, self.base_length())
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        self.cut_at_index(self.bound_iterations_number(size))
    }
}
