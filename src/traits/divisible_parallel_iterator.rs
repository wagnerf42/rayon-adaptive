use crate::prelude::*;
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
    fn wrap_iter(self) -> DivisibleIter<Wrapper<Self>> {
        DivisibleIter {
            base: Wrapper { inner_iter: self },
        }
    }
}

pub struct DivisibleIter<I> {
    pub(crate) base: I,
}

pub struct Wrapper<T> {
    inner_iter: T,
}

impl<T: DivisibleParallelIterator> DivisibleParallelIterator for Wrapper<T> {
    fn base_length(&self) -> usize {
        self.inner_iter.base_length()
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

impl<I> Powered for DivisibleIter<I> {
    type Power = Indexed;
}

impl<I: IntoIterator> ItemProducer for DivisibleIter<I>
where
    I::Item: Sized + Send,
{
    type Item = I::Item;
}

impl<'l, I: DivisibleParallelIterator + IntoIterator> SeqBorrowed<'l> for DivisibleIter<I>
where
    I::Item: Sized + Send,
{
    type Iter = I::IntoIter;
}

impl<I: DivisibleParallelIterator> Divisible for DivisibleIter<I> {
    fn should_be_divided(&self) -> bool {
        self.base.base_length() > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mylen = self.base.base_length();
        (
            DivisibleIter {
                base: self.base.cut_at_index(mylen / 2),
            },
            self,
        )
    }
}

impl<'l, I: DivisibleParallelIterator + IntoIterator> ParBorrowed<'l> for DivisibleIter<I>
where
    I::Item: Sized + Send,
{
    type Iter = DivisibleIter<I>;
}

impl<I: DivisibleParallelIterator + IntoIterator> BorrowingParallelIterator for DivisibleIter<I>
where
    I::Item: Sized + Send,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.base.cut_at_index(size).into_iter()
    }
    fn iterations_number(&self) -> usize {
        self.base.base_length()
    }
}

impl<I: DivisibleParallelIterator + IntoIterator> ParallelIterator for DivisibleIter<I>
where
    I::Item: Sized + Send,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        std::cmp::min(size, self.base.base_length())
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        DivisibleIter {
            base: self.base.cut_at_index(self.bound_iterations_number(size)),
        }
    }
}

// TODO: I don't see any solution but manual implem for each type
//
// impl<I: DivisibleParallelIterator + IntoIterator> IntoParallelIterator for I
// where
//     I::Item: Sized + Send,
// {
//     type Iter = DivisibleIter<I>;
//     type Item = I::Item;
//     fn into_par_iter(self) -> Self::Iter {
//         DivisibleIter { base: self }
//     }
// }

impl<I: DivisibleParallelIterator, J: DivisibleParallelIterator> DivisibleParallelIterator
    for (I, J)
{
    fn base_length(&self) -> usize {
        std::cmp::min(self.0.base_length(), self.1.base_length())
    }
    fn cut_at_index(&mut self, index: usize) -> Self {
        (self.0.cut_at_index(index), self.1.cut_at_index(index))
    }
}
