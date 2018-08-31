//! let's rewrite parallel merge before parallel merge sort
extern crate itertools;
extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;

use rand::random;
use rayon_adaptive::{Divisible, EdibleSlice, EdibleSliceMut, Policy};
use rayon_logs::ThreadPoolBuilder;
use std::iter::repeat;

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

fn main() {
    let v: Vec<u32> = (0..10_000_000).collect();
    let mut left = Vec::new();
    let mut right = Vec::new();
    for e in &v {
        if random() { &mut left } else { &mut right }.push(*e);
    }
    println!("{}/{}", left.len(), right.len());
    let mut w = vec![0; 10_000_000];

    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("pool creation failed");
    let log = pool.install(|| {
        fuse(&left, &right, &mut w);
    }).1;
    log.save_svg("merge.svg").expect("saving svg failed");

    assert_eq!(v, w);
}
