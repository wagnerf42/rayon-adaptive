use crate::iter::ParallelMerge;
use crate::prelude::*;
use crate::scheduler::*;
use std::iter::{once, Once};
use std::marker::PhantomData;
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
    fn wrap(self) -> Wrapper<Self> {
        Wrapper { inner_iter: self }
    }
    fn adaptive_iter(self) -> DivisibleIter<Self, Adaptive> {
        DivisibleIter {
            base: self,
            schedule_type: Adaptive {},
        }
    }
    fn non_adaptive_iter(self) -> DivisibleIter<Self, NonAdaptive> {
        DivisibleIter {
            base: self,
            schedule_type: NonAdaptive {},
        }
    }
}

pub struct DivisibleIter<I, J> {
    pub(crate) base: I,
    pub(crate) schedule_type: J,
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

impl<I, J> Powered for DivisibleIter<I, J> {
    type Power = Indexed;
}

impl<I: IntoIterator, J> ItemProducer for DivisibleIter<I, J>
where
    I::Item: Sized + Send,
    J: Schedulable,
{
    type Item = I::Item;
}

impl<'l, J, I: DivisibleParallelIterator + IntoIterator> SeqBorrowed<'l> for DivisibleIter<I, J>
where
    I: IntoIterator,
    I::Item: Sized + Send,
    J: Schedulable,
{
    type Iter = I::IntoIter;
}

impl<I: DivisibleParallelIterator, J: Copy> Divisible for DivisibleIter<I, J> {
    fn should_be_divided(&self) -> bool {
        self.base.base_length() > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mylen = self.base.base_length();
        (
            DivisibleIter {
                base: self.base.cut_at_index(mylen / 2),
                schedule_type: self.schedule_type,
            },
            self,
        )
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        (
            DivisibleIter {
                base: self.base.cut_at_index(index),
                schedule_type: self.schedule_type,
            },
            self,
        )
    }
}

impl<'l, J, I> ParBorrowed<'l> for DivisibleIter<I, J>
where
    I: DivisibleParallelIterator + IntoIterator,
    I::Item: Sized + Send,
    J: Schedulable + Send + Copy,
{
    type Iter = DivisibleIter<I, J>;
}

impl<J, I> BorrowingParallelIterator for DivisibleIter<I, J>
where
    I: DivisibleParallelIterator + IntoIterator,
    I::Item: Sized + Send,
    J: Schedulable + Send + Copy,
{
    type ScheduleType = J;
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.base.cut_at_index(size).into_iter()
    }
    fn iterations_number(&self) -> usize {
        self.base.base_length()
    }
}

impl<J, I> ParallelIterator for DivisibleIter<I, J>
where
    I: DivisibleParallelIterator + IntoIterator,
    I::Item: Sized + Send,
    J: Schedulable + Send + Copy,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        std::cmp::min(size, self.base.base_length())
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        DivisibleIter {
            base: self.base.cut_at_index(self.bound_iterations_number(size)),
            schedule_type: { self.schedule_type },
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

impl<'a, T: 'a, J> std::ops::Index<usize> for DivisibleIter<&'a [T], J> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &self.base.get_unchecked(index) }
    }
}

impl<I, S> DivisibleIter<I, S>
where
    I: DivisibleParallelIterator + IntoIterator,
    Self: std::ops::Index<usize>,
    <Self as std::ops::Index<usize>>::Output: Ord,
{
    /// Merge two ordered parallel iterators into one ordered parallel iterator.
    pub fn merge<J>(self, other: DivisibleIter<J, S>) -> ParallelMerge<I, J, S>
    where
        J: DivisibleParallelIterator + IntoIterator<Item = I::Item>,
        DivisibleIter<J, S>:
            std::ops::Index<usize, Output = <Self as std::ops::Index<usize>>::Output>,
    {
        ParallelMerge { i: self, j: other }
    }
}
