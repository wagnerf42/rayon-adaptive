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

struct LeftLeft<I, J> {
    i: DivisibleIter<I>,
    j: DivisibleIter<J>,
}

struct RightRight<'niq, I, J> {
    i: DislocatedMut<'niq, DivisibleIter<I>>,
    j: DislocatedMut<'niq, DivisibleIter<J>>,
}

pub struct BorrowingParallelMerge<'par, I, J> {
    ij: either::Either<LeftLeft<I, J>, RightRight<'par, I, J>>,
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

impl<'par, I: 'par, J: 'par> ParBorrowed<'par> for ParallelMerge<I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
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

impl<'afterlife, I: 'afterlife, J: 'afterlife> ParallelIterator for ParallelMerge<I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
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
            ij: either::Right(RightRight {
                i: DislocatedMut::new(&mut self.i),
                j: DislocatedMut::new(&mut self.j),
            }),
            size,
        }
    }
}

impl<'par, I: 'par, J: 'par> BorrowingParallelIterator for BorrowingParallelMerge<'par, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    fn iterations_number(&self) -> usize {
        self.size
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.size -= size;
        let (i, j) = self.ij.as_mut().either(
            |owned| {
                (
                    DislocatedMut::new(&mut owned.i),
                    DislocatedMut::new(&mut owned.j),
                )
            },
            |borrowed| (borrowed.i.borrow_mut(), borrowed.j.borrow_mut()),
        );
        SequentialMerge { i: i, j: j }.take(size)
    }
}

//TODO have to specialise for slices with SliceIndex, get_unchecked is way faster
impl<'par, I: 'par, J: 'par> Divisible for BorrowingParallelMerge<'par, I, J>
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
    I::Item: Send,
    J: DivisibleParallelIterator + IntoIterator<Item = I::Item> + Sync,
    DivisibleIter<J>: Index<usize, Output = <DivisibleIter<I> as Index<usize>>::Output>,
{
    fn should_be_divided(&self) -> bool {
        false // we force the use of the adaptive algorithm ?
    }
    fn divide(mut self) -> (Self, Self) {
        let (mut i_iter, mut j_iter) = self.ij.as_mut().either(
            |owned_stuff| {
                (
                    DislocatedMut::new(&mut owned_stuff.i),
                    DislocatedMut::new(&mut owned_stuff.j),
                )
            },
            |borrowed_stuff| (borrowed_stuff.i.borrow_mut(), borrowed_stuff.j.borrow_mut()),
        );
        let i_len = i_iter.iterations_number();
        let j_len = j_iter.iterations_number();
        let (left_i_diviter, left_j_diviter) = if i_len < j_len {
            //divide j into two and binary search in i
            let left_j_diviter = DivisibleIter {
                base: j_iter.base.cut_at_index(j_len / 2),
            };
            //j_iter is now right side of the cut
            let pivot_value = &left_j_diviter[(j_len - 1) / 2]; // pivot here means the last element of the left side of the cut
            let mut start_index = 0;
            let mut end_index = i_len - 1;
            while end_index != start_index {
                let mid = (start_index + end_index) / 2;
                if i_iter[mid] <= *pivot_value {
                    //right side of the binary-search cut should not have equal values.
                    start_index = mid + 1
                } else {
                    end_index = mid - 1
                }
            }
            while i_iter[start_index] == *pivot_value && start_index < i_len {
                start_index += 1;
            }
            let left_i_diviter = DivisibleIter {
                base: i_iter.base.cut_at_index(start_index),
            }; //Left side does not include start_index, cut should not include index on the left
            (left_i_diviter, left_j_diviter)
        } else {
            //divide i into two and binary search in j
            let left_i_diviter = DivisibleIter {
                base: i_iter.base.cut_at_index(i_len / 2),
            };
            //i_iter is now right side of the cut
            let pivot_value = &left_i_diviter[(i_len - 1) / 2]; // pivot here means the last element of the left side of the cut
            let mut start_index = 0;
            let mut end_index = j_len - 1;
            while end_index != start_index {
                let mid = (start_index + end_index) / 2;
                if j_iter[mid] <= *pivot_value {
                    start_index = mid + 1
                } else {
                    end_index = mid - 1
                }
            }
            while j_iter[start_index] == *pivot_value && start_index < j_len {
                start_index += 1;
            }
            let left_j_diviter = DivisibleIter {
                base: j_iter.base.cut_at_index(start_index),
            }; //Left side does not include start_index, cut should not include index on the left
            (left_i_diviter, left_j_diviter)
        };
        let left_div_size = left_i_diviter.iterations_number() + left_j_diviter.iterations_number();
        let right_div_size = i_iter.iterations_number() + j_iter.iterations_number();
        let left_div: either::Either<LeftLeft<I, J>, RightRight<I, J>> = either::Left(LeftLeft {
            i: left_i_diviter,
            j: left_j_diviter,
        });
        let right_div = self.ij.either(
            |owned_stuff| {
                either::Left(LeftLeft {
                    i: owned_stuff.i,
                    j: owned_stuff.j,
                })
            },
            |borrowed_stuff| {
                either::Right(RightRight {
                    i: borrowed_stuff.i,
                    j: borrowed_stuff.j,
                })
            },
        );
        (
            BorrowingParallelMerge {
                ij: left_div,
                size: left_div_size,
            },
            BorrowingParallelMerge {
                ij: right_div,
                size: right_div_size,
            },
        )
        //let pivot_value = &self.i[0];
        //let mut start_index = 0;
        //let mut end_index = self.j.iterations_number() - 1;
        //while end_index != start_index {
        //    let mid = (start_index + end_index) / 2;
        //    if self.j[mid] <= *pivot_value {
        //        start_index = mid + 1
        //    } else {
        //        end_index = mid - 1
        //    }
        //}
        //while self.j[start_index] == *pivot_value && start_index < self.j.iterations_number() {
        //    start_index += 1;
        //}
        //let left_j_diviter = self.j.base.cut_at_index(start_index); //Left side does not include start_index
        //let left_left = DislocatedMut::new(&mut DivisibleIter {
        //    base: left_i_diviter,
        //});
        //let left_right = DislocatedMut::new(&mut DivisibleIter {
        //    base: left_j_diviter,
        //});
        //(
        //    BorrowingParallelMerge {
        //        i: left_left,
        //        j: left_right,
        //        size: self.size / 2 + start_index,
        //    },
        //    BorrowingParallelMerge {
        //        i: self.i,
        //        j: self.j,
        //        size: self.size / 2 + start_index,
        //    },
        //)
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
        let i_is_empty = self.i.completed();
        let j_is_empty = self.j.completed();
        if !i_is_empty && !j_is_empty {
            if self.i[0] <= self.j[0] {
                self.i.next()
            } else {
                self.j.next()
            }
        } else if j_is_empty {
            self.i.next()
        } else {
            self.j.next()
        }
    }
}
