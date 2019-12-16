use crate::dislocated::DislocatedMut;
/// Implementation of a two way ordered merge for ParallelIterators.
/// This code was initially developed by Louis Boulanger.
use crate::prelude::*;
use std::iter::Take;
use std::marker::PhantomData;
use std::ops::Index;

/// Parallel iterator obtained by merging ordered parallel iterators.
/// Both iterators need to have a peeking ability.
/// Obtained from the `merge` method on `PeekableIterator`.
pub struct ParallelMerge<T, I, J> {
    pub(crate) left: I,
    pub(crate) right: J,
    pub(crate) phantom: PhantomData<T>, // we need this to store peekedType
}

impl<T, I, J> ItemProducer for ParallelMerge<T, I, J>
where
    I: ItemProducer,
{
    type Item = I::Item;
}

impl<T, I, J> Powered for ParallelMerge<T, I, J> {
    type Power = Indexed;
}

impl<'e, T, I, J> ParBorrowed<'e> for ParallelMerge<T, I, J>
where
    T: Ord + Send + 'static, // it's ugly but we need this static or a dislocation
    I: ParallelIterator,
    for<'f> <I as ParBorrowed<'f>>::Iter: Index<usize, Output = T>,
    J: ParallelIterator<Item = I::Item>,
    for<'f> <J as ParBorrowed<'f>>::Iter: Index<usize, Output = T>,
{
    type Iter = ParallelMerge<T, <I as ParBorrowed<'e>>::Iter, <J as ParBorrowed<'e>>::Iter>;
}

impl<'e, T, I, J> SeqBorrowed<'e> for ParallelMerge<T, I, J>
where
    T: Ord + Send + 'static,
    I: BorrowingParallelIterator + Index<usize, Output = T>,
    J: BorrowingParallelIterator<Item = I::Item> + Index<usize, Output = T>,
{
    type Iter = Take<SequentialMerge<'e, I, J>>;
}

impl<T, I, J> ParallelIterator for ParallelMerge<T, I, J>
where
    T: Ord + Send + 'static,
    I: ParallelIterator,
    for<'f> <I as ParBorrowed<'f>>::Iter: Index<usize, Output = T>,
    J: ParallelIterator<Item = I::Item>,
    for<'f> <J as ParBorrowed<'f>>::Iter: Index<usize, Output = T>,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        unimplemented!()
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        let (left, right) = (self.left.par_borrow(size), self.right.par_borrow(size));
        ParallelMerge {
            left,
            right,
            phantom: PhantomData,
        }
    }
}

impl<T, I, J> BorrowingParallelIterator for ParallelMerge<T, I, J>
where
    T: Ord + Send + 'static,
    I: BorrowingParallelIterator + Index<usize, Output = T>,
    J: BorrowingParallelIterator<Item = I::Item> + Index<usize, Output = T>,
{
    fn iterations_number(&self) -> usize {
        self.left.iterations_number() + self.right.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        unimplemented!()
    }
}

impl<T, I, J> Divisible for ParallelMerge<T, I, J>
where
    T: Ord + Send + 'static,
    I: BorrowingParallelIterator + Index<usize, Output = T>,
    J: BorrowingParallelIterator + Index<usize, Output = T>,
{
    fn should_be_divided(&self) -> bool {
        unimplemented!()
    }
    fn divide(self) -> (Self, Self) {
        unimplemented!()
    }
}

/// Sequential 2-way Merge struct obtained from parallel 2-way merge.
pub struct SequentialMerge<'e, I, J> {
    left: DislocatedMut<'e, I>,
    right: DislocatedMut<'e, J>,
}

impl<'e, I, J> Iterator for SequentialMerge<'e, I, J>
where
    I: BorrowingParallelIterator + Index<usize>,
    I::Output: Ord,
    J: BorrowingParallelIterator<Item = I::Item> + Index<usize, Output = I::Output>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let left_is_empty = self.left.completed();
        let right_is_empty = self.right.completed();
        if !left_is_empty && !right_is_empty {
            if self.left[0] <= self.right[0] {
                self.left.next()
            } else {
                self.right.next()
            }
        } else if right_is_empty {
            self.left.next()
        } else {
            self.right.next()
        }
    }
}
