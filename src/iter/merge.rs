//! Implementation of a two way ordered merge for ParallelIterators.
//! This code was initially developed by Louis Boulanger.
//!
//! There are lots of things to re-think here.
//!
//! - power level is between indexed and medium (you can take but not zip)
//! - you could theoretically have unbounded ranges merged together
//! - what if you map (do you call op several times for one value if peeked ?)
//! - the semantic for par_borrow (in general, not here) is not completely homogeneous:
//!    * sometimes when we par_borrow for size n we eat n even if the iterator is not consumed
//!    * sometimes when just eat whatever size we consume
//!
use crate::dislocated::DislocatedMut;
use crate::prelude::*;
use std::iter::Take;
use std::ops::Index;

/// Parallel iterator obtained by merging ordered parallel iterators.
/// Both iterators need to have a peeking ability.
/// Obtained from the `merge` method on `PeekableIterator`.
/// We technically should be able to do any parallel iterator
/// but I have some trouble with infinite iterators (either I cannot divide
/// or I cannot par_borrow).
/// For now we start with only DivisibleIter.
pub struct ParallelMerge<I, J> {
    pub(crate) i: DivisibleIter<I>,
    pub(crate) j: DivisibleIter<J>,
}

pub struct BorrowingParallelMerge<'par, I, J> {
    i: DislocatedMut<'par, DivisibleIter<I>>,
    j: DislocatedMut<'par, DivisibleIter<J>>,
    size: usize,
}

impl<I, J> ItemProducer for ParallelMerge<I, J>
where
    I: IntoIterator,
    I::Item: Send,
{
    type Item = I::Item;
}

impl<'par, I, J> ItemProducer for BorrowingParallelMerge<'par, I, J>
where
    I: IntoIterator,
    I::Item: Send,
{
    type Item = I::Item;
}

impl<I, J> Powered for ParallelMerge<I, J> {
    type Power = Indexed;
}

impl<'par, I, J> ParBorrowed<'par> for ParallelMerge<I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    type Iter = BorrowingParallelMerge<'par, I, J>;
}

impl<'par, 'seq, I, J> SeqBorrowed<'seq> for BorrowingParallelMerge<'par, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    type Iter = Take<SequentialMerge<'seq, I, J>>;
}

impl<I, J> ParallelIterator for ParallelMerge<I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        let available = self.i.base.base_length() + self.j.base.base_length();
        std::cmp::min(available, size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowingParallelMerge {
            i: DislocatedMut::new(&mut self.i),
            j: DislocatedMut::new(&mut self.j),
            size,
        }
    }
}

impl<'par, I, J> BorrowingParallelIterator for BorrowingParallelMerge<'par, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    fn iterations_number(&self) -> usize {
        self.size
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.size -= size;
        SequentialMerge {
            i: self.i.borrow_mut(),
            j: self.j.borrow_mut(),
        }
        .take(size)
    }
}

impl<'par, I, J> Divisible for BorrowingParallelMerge<'par, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    fn should_be_divided(&self) -> bool {
        false // we force the use of the adaptive algorithm ?
    }
    fn divide(self) -> (Self, Self) {
        unimplemented!()
        //        if self.i.iterations_number() <= self.j.iterations_number() {
        //            let (left_i, right_i) = self.i.divide();
        //            // we take the pivot as the last element of left side.
        //            // this way we are sure there is always one.
        //            let pivot_index = left_i.iterations_number() - 1;
        //            let pivot_value = &left_i[pivot_index];
        //            // do a binary search on j
        //            let mut start_index = 0;
        //            let mut end_index = self.j.iterations_number() - 1;
        //            while end_index != start_index {
        //                let mid = (start_index + end_index) / 2;
        //                if &self.j[mid] == pivot_value {
        //                    unimplemented!()
        //                }
        //                if &self.j[mid] < pivot_value {
        //                    start_index = mid + 1
        //                } else {
        //                    end_index = mid - 1
        //                }
        //            }
        //            unimplemented!()
        //        } else {
        //            unimplemented!()
        //        }
    }
}

/// Sequential 2-way Merge struct obtained from parallel 2-way merge.
pub struct SequentialMerge<'e, I, J> {
    i: DislocatedMut<'e, DivisibleIter<I>>,
    j: DislocatedMut<'e, DivisibleIter<J>>,
}

impl<'e, I, J> Iterator for SequentialMerge<'e, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
        //        let i_is_empty = self.i.completed();
        //        let j_is_empty = self.j.completed();
        //        if !i_is_empty && !j_is_empty {
        //            if self.i[0] <= self.j[0] {
        //                self.i.next()
        //            } else {
        //                self.j.next()
        //            }
        //        } else if j_is_empty {
        //            self.i.next()
        //        } else {
        //            self.j.next()
        //        }
    }
}
