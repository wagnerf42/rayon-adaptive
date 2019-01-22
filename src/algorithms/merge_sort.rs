//! adaptive parallel merge sort.
use crate::prelude::*;
use crate::traits::{BasicPower, BlockedPower};
use crate::{fuse_slices, EdibleSlice, EdibleSliceMut, Policy};
use std;
use std::cmp::min;
use std::iter::repeat;

// main related code

/// find subslice without last value in given sorted slice.
fn subslice_without_last_value<T: Eq>(slice: &[T]) -> &[T] {
    match slice.split_last() {
        Some((target, slice)) => {
            let searching_range_start = repeat(())
                .scan(1, |acc, _| {
                    *acc *= 2;
                    Some(*acc)
                }) // iterate on all powers of 2
                .take_while(|&i| i < slice.len())
                .map(|i| slice.len() - i) // go farther and farther from end of slice
                .find(|&i| unsafe { slice.get_unchecked(i) != target })
                .unwrap_or(0);

            let index = slice[searching_range_start..]
                .binary_search_by(|x| {
                    if x.eq(target) {
                        std::cmp::Ordering::Greater
                    } else {
                        std::cmp::Ordering::Less
                    }
                })
                .unwrap_err();
            &slice[0..(searching_range_start + index)]
        }
        None => slice,
    }
}

/// find subslice without first value in given sorted slice.
fn subslice_without_first_value<T: Eq>(slice: &[T]) -> &[T] {
    match slice.first() {
        Some(target) => {
            let searching_range_end = repeat(())
                .scan(1, |acc, _| {
                    *acc *= 2;
                    Some(*acc)
                }) // iterate on all powers of 2
                .take_while(|&i| i < slice.len())
                .find(|&i| unsafe { slice.get_unchecked(i) != target })
                .unwrap_or_else(|| slice.len());

            let index = slice[..searching_range_end]
                .binary_search_by(|x| {
                    if x.eq(target) {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                })
                .unwrap_err();
            &slice[index..]
        }
        None => slice,
    }
}

/// Cut sorted slice `slice` around start point, splitting around
/// all values equal to value at start point.
/// cost is O(log(|removed part size|))
fn split_around<T: Eq>(slice: &[T], start: usize) -> (&[T], &[T], &[T]) {
    let low_slice = subslice_without_last_value(&slice[0..=start]);
    let high_slice = subslice_without_first_value(&slice[start..]);
    let equal_slice = &slice[low_slice.len()..slice.len() - high_slice.len()];
    (low_slice, equal_slice, high_slice)
}

/// split large array at midpoint and small array where needed for merge.
fn merge_split<'a, T: Ord>(
    large: &'a [T],
    small: &'a [T],
) -> ((&'a [T], &'a [T], &'a [T]), (&'a [T], &'a [T], &'a [T])) {
    let middle = large.len() / 2;
    let split_large = split_around(large, middle);
    let split_small = match small.binary_search(&large[middle]) {
        Ok(i) => split_around(small, i),
        Err(i) => {
            let (small1, small3) = small.split_at(i);
            (small1, &small[0..0], small3)
        }
    };
    (split_large, split_small)
}

struct FusionSlice<'a, T: 'a> {
    left: EdibleSlice<'a, T>,
    right: EdibleSlice<'a, T>,
    output: EdibleSliceMut<'a, T>,
}

impl<'a, T: 'a + Send + Sync + Ord + Copy> Divisible for FusionSlice<'a, T> {
    type Power = BasicPower;
    fn base_length(&self) -> usize {
        self.output.base_length()
    }
    fn divide(self) -> (Self, Self) {
        let left = self.left.remaining_slice();
        let right = self.right.remaining_slice();
        let output = self.output.into_remaining_slice();
        let ((l1, l2, l3), (r1, r2, r3)) = if left.len() > right.len() {
            merge_split(left, right)
        } else {
            let (r, l) = merge_split(right, left);
            (l, r)
        };
        let (o1, ol) = output.split_at_mut(l1.len() + r1.len());
        let (o2, o3) = ol.split_at_mut(l2.len() + r2.len());
        // immediately copy sequentially the middle part
        o2[..l2.len()].copy_from_slice(l2);
        o2[l2.len()..].copy_from_slice(r2);
        // return what is left to do
        (
            FusionSlice {
                left: EdibleSlice::new(l1),
                right: EdibleSlice::new(r1),
                output: EdibleSliceMut::new(o1),
            },
            FusionSlice {
                left: EdibleSlice::new(l3),
                right: EdibleSlice::new(r3),
                output: EdibleSliceMut::new(o3),
            },
        )
    }
}

fn fuse<T: Ord + Send + Sync + Copy>(left: &[T], right: &[T], output: &mut [T], policy: Policy) {
    let slices = FusionSlice {
        left: EdibleSlice::new(left),
        right: EdibleSlice::new(right),
        output: EdibleSliceMut::new(output),
    };

    slices
        .with_policy(policy)
        .partial_for_each(|mut slices, limit| {
            {
                let mut left_i = slices.left.iter();
                let mut right_i = slices.right.iter();
                for o in slices.output.iter_mut().take(limit) {
                    let go_left = match (left_i.peek(), right_i.peek()) {
                        (Some(l), Some(r)) => l <= r,
                        (Some(_), None) => true,
                        (None, Some(_)) => false,
                        (None, None) => panic!("not enough input when merging"),
                    };
                    *o = if go_left {
                        *left_i.next().unwrap()
                    } else {
                        *right_i.next().unwrap()
                    };
                }
            }
            slices
        });
}

// sort related code

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>,
    i: usize,
}

impl<'a, T: 'a + Ord + Sync + Copy + Send> SortingSlices<'a, T> {
    /// Call parallel merge on the right slices.
    fn fuse_with_policy(self, other: Self, policy: Policy) -> Self {
        let mut left = self;
        let mut right = other;
        // let's try a nice optimization here for nearly sorted arrays.
        // if slices are already sorted and at same index then we do nothing !
        let destination_index = if left.i == right.i
            && left.s[left.i].last() <= right.s[right.i].first()
        {
            left.i
        } else {
            let destination_index = (0..3).find(|&x| x != left.i && x != right.i).unwrap();
            {
                let left_index = left.i;
                let right_index = right.i;
                let (left_input, left_output) = left.mut_couple(left_index, destination_index);
                let (right_input, right_output) = right.mut_couple(right_index, destination_index);
                let output_slice = fuse_slices(left_output, right_output);
                // if slices are nearly sorted we will resort to memcpy
                if left_input.last() <= right_input.first() {
                    output_slice[..left_input.base_length()].copy_from_slice(left_input);
                    output_slice[left_input.base_length()..].copy_from_slice(right_input);
                } else if right_input.last() < left_input.first() {
                    output_slice[..right_input.base_length()].copy_from_slice(right_input);
                    output_slice[right_input.base_length()..].copy_from_slice(left_input);
                } else {
                    fuse(left_input, right_input, output_slice, policy);
                }
            }
            destination_index
        };
        let fused_slices: Vec<_> = left
            .s
            .into_iter()
            .zip(right.s.into_iter())
            .map(|(left_s, right_s)| fuse_slices(left_s, right_s))
            .collect();
        SortingSlices {
            s: fused_slices,
            i: destination_index,
        }
    }

    /// Borrow all mutable slices at once.
    fn mut_slices(&mut self) -> (&mut [T], &mut [T], &mut [T]) {
        let (s0, leftover) = self.s.split_first_mut().unwrap();
        let (s1, s2) = leftover.split_first_mut().unwrap();
        (s0, s1, s2[0])
    }
    /// Return the two mutable slices of given indices.
    fn mut_couple(&mut self, i1: usize, i2: usize) -> (&mut [T], &mut [T]) {
        let (s0, s1, s2) = self.mut_slices();
        match (i1, i2) {
            (0, 1) => (s0, s1),
            (0, 2) => (s0, s2),
            (1, 0) => (s1, s0),
            (1, 2) => (s1, s2),
            (2, 0) => (s2, s0),
            (2, 1) => (s2, s1),
            _ => panic!("i1 == i2"),
        }
    }
    fn split_at(self, i: usize) -> (Self, Self) {
        let v = self.s.into_iter().map(|s| s.split_at_mut(i)).fold(
            (Vec::new(), Vec::new()),
            |mut acc, (s1, s2)| {
                acc.0.push(s1);
                acc.1.push(s2);
                acc
            },
        );
        (
            SortingSlices { s: v.0, i: self.i },
            SortingSlices { s: v.1, i: self.i },
        )
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Divisible for SortingSlices<'a, T> {
    type Power = BlockedPower;
    fn base_length(&self) -> usize {
        self.s[0].base_length()
    }
    fn divide(self) -> (Self, Self) {
        let mid = self.s[0].base_length() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> DivisibleIntoBlocks for SortingSlices<'a, T> {
    fn divide_at(self, i: usize) -> (Self, Self) {
        self.split_at(i)
    }
}

/// Sort given slice using an adaptive version of merge sort.
/// For now we require Copy on T.
/// Sort is stable.
///
/// # Examples
///
/// ```
/// use rayon_adaptive::adaptive_sort;
/// let v: Vec<u32> = (0..100_000).collect();
/// let mut inverted_v: Vec<u32> = (0..100_000).rev().collect();
/// adaptive_sort(&mut inverted_v);
/// assert_eq!(v, inverted_v);
/// ```
pub fn adaptive_sort<T: Ord + Copy + Send + Sync + std::fmt::Debug>(slice: &mut [T]) {
    let mut tmp_slice1 = Vec::with_capacity(slice.base_length());
    let mut tmp_slice2 = Vec::with_capacity(slice.base_length());
    unsafe {
        tmp_slice1.set_len(slice.base_length());
        tmp_slice2.set_len(slice.base_length());
    }

    let slices = SortingSlices {
        s: vec![slice, tmp_slice1.as_mut_slice(), tmp_slice2.as_mut_slice()],
        i: 0,
    };

    let mut result_slices = slices.map_reduce(
        |mut slices| {
            slices.s[slices.i].sort();
            slices
        },
        |s1, s2| s1.fuse_with_policy(s2, Default::default()),
    );

    if result_slices.i != 0 {
        let i = result_slices.i;
        let (destination, source) = result_slices.mut_couple(0, i);
        destination.copy_from_slice(source);
    }
}
