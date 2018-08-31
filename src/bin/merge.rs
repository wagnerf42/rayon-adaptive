//! let's rewrite parallel merge before parallel merge sort
extern crate itertools;
extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;

use rand::random;
use rayon_adaptive::{Divisible, EdibleSlice, EdibleSliceMut, Mergeable, Policy};
use rayon_logs::ThreadPoolBuilder;
use std::cmp::min;
use std::iter::repeat;

// main related code

/// find subslice without last value in given sorted slice.
fn subslice_without_last_value<T: Eq>(slice: &[T]) -> &[T] {
    match slice.split_last() {
        Some((target, slice)) => {
            let searching_range_start = repeat(())
        .scan(1, |acc, _| {*acc *= 2 ; Some(*acc)}) // iterate on all powers of 2
        .take_while(|&i| i < slice.len())
        .map(|i| slice.len() -i) // go farther and farther from end of slice
        .find(|&i| unsafe {slice.get_unchecked(i) != target})
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
        .scan(1, |acc, _| {*acc *= 2; Some(*acc)}) // iterate on all powers of 2
        .take_while(|&i| i < slice.len())
        .find(|&i| unsafe {slice.get_unchecked(i) != target})
        .unwrap_or_else(||slice.len());

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
    let low_slice = subslice_without_last_value(&slice[0..(start + 1)]);
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
    fn len(&self) -> usize {
        self.output.len()
    }
    fn split(self) -> (Self, Self) {
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

fn sequential_fuse<T: Ord + Copy>(left: &[T], right: &[T], output: &mut [T]) {
    let mut left_iterator = left.iter().peekable();
    let mut right_iterator = right.iter().peekable();
    for o in output {
        let go_left = match (left_iterator.peek(), right_iterator.peek()) {
            (Some(l), Some(r)) => l <= r,
            (Some(_), None) => true,
            (None, Some(_)) => false,
            (None, None) => panic!("not enough input when merging"),
        };
        *o = if go_left {
            *left_iterator.next().unwrap()
        } else {
            *right_iterator.next().unwrap()
        };
    }
}

fn fuse<T: Ord + Send + Sync + Copy>(left: &[T], right: &[T], output: &mut [T]) {
    let slices = FusionSlice {
        left: EdibleSlice::new(left),
        right: EdibleSlice::new(right),
        output: EdibleSliceMut::new(output),
    };

    slices.work(
        |slices, limit| {
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
        },
        |_| (),
        Policy::Adaptive(10_000),
    );
}

// sort related code

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>, // we have 2 input slices and one output slice
    i: usize,            // index of slice containing the data
    eaten: usize,        // how much has already been processed
}

impl<'a, T: 'a> SortingSlices<'a, T> {
    /// Return the two mutable slices of given indices.
    fn mut_couple<'b>(&'b mut self, i1: usize, i2: usize) -> (&'b mut [T], &'b mut [T]) {
        let (s0, s1, s2) = {
            let (s0, leftover) = self.s.split_first_mut().unwrap();
            let (s1, s2) = leftover.split_first_mut().unwrap();
            (s0, s1, s2.get_mut(0).unwrap())
        };
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
            SortingSlices {
                s: v.0,
                i: self.i,
                eaten: self.eaten,
            },
            SortingSlices {
                s: v.1,
                i: self.i,
                eaten: 0,
            },
        )
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Divisible for SortingSlices<'a, T> {
    fn len(&self) -> usize {
        self.s[0].len() - self.eaten
    }
    fn split(self) -> (Self, Self) {
        let mid = (self.s[0].len() - self.eaten) / 2;
        let eaten = self.eaten;
        self.split_at(eaten + mid)
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Mergeable for SortingSlices<'a, T> {
    fn fuse(self, other: Self) -> Self {
        unimplemented!()
    }
}

fn adaptive_sort<T: Ord + Copy + Send + Sync>(slice: &mut [T]) {
    let mut tmp_slice1 = Vec::with_capacity(slice.len());
    let mut tmp_slice2 = Vec::with_capacity(slice.len());
    unsafe {
        tmp_slice1.set_len(slice.len());
        tmp_slice2.set_len(slice.len());
    }

    let slices = SortingSlices {
        s: vec![slice, tmp_slice1.as_mut_slice(), tmp_slice2.as_mut_slice()],
        i: 0,
        eaten: 0,
    };

    let result_slices = slices.work(
        |slices, limit| {
            let eaten = slices.eaten;
            let new_eaten = min(slices.eaten + limit, slices.s[0].len());
            let input_index = slices.i;
            let output_index = (input_index + 1) % 3;
            {
                let (in_slice, out_slice) = slices.mut_couple(input_index, output_index);
                in_slice[eaten..new_eaten].sort();
                sequential_fuse(
                    &in_slice[0..eaten],
                    &in_slice[eaten..new_eaten],
                    &mut out_slice[0..new_eaten],
                );
            }
            slices.i = output_index;
            slices.eaten = new_eaten;
        },
        |s| s,
        Policy::Adaptive(2_000),
    );

    unimplemented!()
}

fn main() {
    let mut v: Vec<u32> = (0..1_000_000).map(|_| random::<u32>() % 100_000).collect();
    let mut w = v.clone();
    w.sort();
    adaptive_sort(&mut v);

    let pool = ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool creation failed");
    let log = pool.install(|| {
        adaptive_sort(&mut v);
    }).1;
    log.save_svg("merge.svg").expect("saving svg failed");

    assert_eq!(v, w);
}

// fn main() {
//
//     let v: Vec<u32> = (0..10_000_000).collect();
//     let mut left = Vec::new();
//     let mut right = Vec::new();
//     for e in &v {
//         if random() { &mut left } else { &mut right }.push(*e);
//     }
//     println!("{}/{}", left.len(), right.len());
//     let mut w = vec![0; 10_000_000];
//
//     let pool = ThreadPoolBuilder::new()
//         .num_threads(4)
//         .build()
//         .expect("pool creation failed");
//     let log = pool.install(|| {
//         fuse(&left, &right, &mut w);
//     }).1;
//     log.save_svg("merge.svg").expect("saving svg failed");
//
//     assert_eq!(v, w);
// }
