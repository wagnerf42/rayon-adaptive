use crate::prelude::{
    BorrowingParallelIterator, Divisible, ParBorrowed, ParallelIterator, Powered, SeqBorrowed,
};
use crate::traits::Indexed;
/// This trait provides a shortcut to making a parallel iterator. If this trait is implemented, the
/// type shall also automatically implement ParallelIterator and IndexedParallelIterator traits.
/// The catch is that even while making macro blocks, this divide_at_index() method will be called.
/// If the divide_at_index() method is expensive, go the longer route and implement
/// ParallelIterator yourself.
pub trait DivisibleParallelIterator: Send + Sized + IntoIterator {
    fn base_length(&self) -> usize;
    /// Cuts self, left side is returned and self is now the right side of the cut
    fn cut_at_index(&mut self, index: usize) -> Self;
}

impl<I: DivisibleParallelIterator> Powered for I {
    type Power = Indexed;
}

impl<'l, I: DivisibleParallelIterator> SeqBorrowed<'l> for I
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

impl<'l, I: DivisibleParallelIterator> ParBorrowed<'l> for I
where
    I::Item: Sized + Send,
{
    type Iter = I;
}

impl<I: DivisibleParallelIterator> BorrowingParallelIterator for I
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

impl<I: DivisibleParallelIterator> ParallelIterator for I
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
