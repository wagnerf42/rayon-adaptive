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

impl<'par, I, J> ParBorrowed<'par> for ParallelMerge<I, J>
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

impl<I, J> ParallelIterator for ParallelMerge<I, J>
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

impl<'par, I, J> BorrowingParallelIterator for BorrowingParallelMerge<'par, I, J>
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
    fn part_completed(&self) -> bool {
        //In the case of the merge this has to serve as a triviality check.
        //If the left and right ends match, we don't divide and let sequential iter take over.
        self.iterations_number() == 0
            || self.ij.as_ref().either(
                |left| left.i.iterations_number() < 4 || left.j.iterations_number() < 4,
                |right| right.i.iterations_number() < 4 || right.j.iterations_number() < 4,
            )
            || self.ij.as_ref().either(
                |left| {
                    left.i[left.i.iterations_number() - 1] <= left.j[0]
                        || left.j[left.j.iterations_number() - 1] <= left.i[0]
                },
                |right| {
                    right.i[right.i.iterations_number() - 1] <= right.j[0]
                        || right.j[right.j.iterations_number() - 1] <= right.i[0]
                },
            )
    }
}

//Cut sorted_iter into two roughly equal pieces.
//It keeps the right side in sorted_iter and returns the left side
//
//The tricky part is that sorted_iter[len/2] might be repeated to the left and/or to the right.
//If so, we search with exponential growth, on the left and right side of the center for a different
//value.
//As soon as we find a unique value on either side, we do a binary search between that unique value
//and the middle value, to find the extent of the repetition.
//
//If we had searched on the left side, we keep the repeated values on the right side of the cut.
//Else if we had searched on the right side, we keep the repeated values on the left side of the
//cut.
//
//PRECONDITION: sorted_iter can not have all values equal
//This should be unit-tested
fn cut_around_middle<I>(sorted_iter: &mut DivisibleIter<I>) -> I
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
{
    let iter_len = sorted_iter.base.base_length();
    if sorted_iter[iter_len / 2] != sorted_iter[iter_len / 2 + 1] {
        sorted_iter.base.cut_at_index(iter_len / 2 + 1)
    } else if sorted_iter[iter_len / 2] != sorted_iter[iter_len / 2 - 1] {
        sorted_iter.base.cut_at_index(iter_len / 2)
    } else {
        let middle_value = &sorted_iter[iter_len / 2];
        //This is not good for the cache, but will make the search direction agnostic
        let first_unequal_position = (1..)
            .map(|power: u32| 2_usize.pow(power))
            .take_while(|&power_of_two| power_of_two <= (iter_len - 1) / 2)
            .take_while(|power_of_two| {
                middle_value != &sorted_iter[iter_len / 2 + power_of_two]
                    || middle_value != &sorted_iter[iter_len / 2 - power_of_two]
            })
            .last()
            .unwrap();
        if middle_value != &sorted_iter[iter_len / 2 - first_unequal_position] {
            let mut start = iter_len / 2 - first_unequal_position;
            let mut end = iter_len / 2 - (first_unequal_position >> 1); //search only till the previous power of two
            while start < end - 1 {
                let mid = (start + end) / 2;
                if &sorted_iter[mid] < middle_value {
                    //LOOP INVARIANT sorted_iter[start] < middle_value
                    start = mid;
                } else {
                    end = mid;
                }
            }
            debug_assert!(start == end - 1);
            sorted_iter.base.cut_at_index(end)
        } else if middle_value != &sorted_iter[iter_len / 2 + first_unequal_position] {
            let mut start = iter_len / 2 + (first_unequal_position >> 1);
            let mut end = iter_len / 2 + first_unequal_position;
            while start < end - 1 {
                let mid = (start + end) / 2;
                if &sorted_iter[mid] <= middle_value {
                    //LOOP INVARIANT sorted_iter[end] > middle_value
                    start = mid;
                } else {
                    end = mid;
                }
            }
            debug_assert!(start == end - 1);
            sorted_iter.base.cut_at_index(end)
        } else {
            //I think this is possible only if the length is even
            //and sorted_iter[0] is unique while all others are equal to middle value
            assert!(iter_len % 2 == 1);
            assert!(sorted_iter[1] == sorted_iter[iter_len - 1]);
            assert!(sorted_iter[0] != sorted_iter[1]);
            sorted_iter.base.cut_at_index(1)
        }
    }
}

//This may get all equal values in the toughest case.
//If the repeating value is equal to the search value, we cut sorted_iter
//in half and return.
fn search_and_cut<I>(
    sorted_iter: &mut DivisibleIter<I>,
    value: &<DivisibleIter<I> as Index<usize>>::Output,
) -> I
where
    I: DivisibleParallelIterator + IntoIterator + Sync,
    DivisibleIter<I>: Index<usize>,
    <DivisibleIter<I> as Index<usize>>::Output: Ord + Sized,
{
    let iter_len = sorted_iter.base.base_length();
    if sorted_iter[0] != sorted_iter[iter_len - 1] {
        if &sorted_iter[0] < value {
            //even if value repeats, it surely doesn't repeat till the left end
            let mut start = 0;
            let mut end = iter_len - 1;
            while start < end - 1 {
                let mid = (start + end) / 2;
                if &sorted_iter[mid] < value {
                    start = mid;
                } else {
                    end = mid;
                }
            }
            sorted_iter.base.cut_at_index(end)
        } else if &sorted_iter[iter_len - 1] > value {
            //even if value repeats, it surely doesn't repeat till the right end
            let mut start = 0;
            let mut end = iter_len - 1;
            while start < end - 1 {
                let mid = (start + end) / 2;
                if &sorted_iter[mid] <= value {
                    start = mid;
                } else {
                    end = mid;
                }
            }
            sorted_iter.base.cut_at_index(end)
        } else {
            panic!("it is midnight and I have no clue why this is printed")
        }
    } else {
        if &sorted_iter[0] < value {
            sorted_iter.base.cut_at_index(iter_len)
        } else if &sorted_iter[0] > value {
            sorted_iter.base.cut_at_index(0)
        } else {
            //This would be something!
            sorted_iter.base.cut_at_index(iter_len / 2)
        }
    }
}

impl<'par, I, J> Divisible for BorrowingParallelMerge<'par, I, J>
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
        debug_assert!(i_len > 1 && j_len > 1);
        let (left_i_diviter, left_j_diviter) = if i_len < j_len && j_iter[0] != j_iter[j_len - 1] {
            //j is bigger and does not have the same value repeated.
            //if j has the same value over and over again, i will have some unique stuff and the j
            //value will fit somewhere in between i values (else part_completed would not have
            //allowed division with the triviality check)
            let left_j_diviter = DivisibleIter {
                base: cut_around_middle(&mut j_iter),
            };
            //j_iter is now right side of the cut
            let pivot_value = &left_j_diviter[left_j_diviter.iterations_number() - 1];
            let left_i_diviter = DivisibleIter {
                base: search_and_cut(&mut i_iter, pivot_value),
            }; //Left side does not include start_index, cut should not include index on the left
            (left_i_diviter, left_j_diviter)
        } else {
            //divide i into two and binary search in j
            let left_i_diviter = DivisibleIter {
                base: cut_around_middle(&mut i_iter),
            };
            //i_iter is now right side of the cut
            let pivot_value = &left_i_diviter[left_i_diviter.iterations_number() - 1];
            let left_j_diviter = DivisibleIter {
                base: search_and_cut(&mut j_iter, pivot_value),
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
