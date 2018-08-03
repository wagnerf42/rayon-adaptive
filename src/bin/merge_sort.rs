extern crate itertools;
extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::{ChaChaRng, Rng};
use rayon_logs::ThreadPoolBuilder;

use rayon_adaptive::{Divisible, Mergeable, Policy};
use std::iter::repeat;

trait Boolean {
    fn value() -> bool;
}
struct True;
struct False;
impl Boolean for True {
    fn value() -> bool {
        true
    }
}
impl Boolean for False {
    fn value() -> bool {
        false
    }
}

fn partial_manual_merge<
    CheckLeft: Boolean,
    CheckRight: Boolean,
    CheckLimit: Boolean,
    T: Ord + Copy,
>(
    input1: &[T],
    input2: &[T],
    output: &mut [T],
    limit: usize,
) -> Option<(usize, usize, usize)> {
    let mut i1 = 0;
    let mut i2 = 0;
    let mut i_out = 0;
    let (check_left, check_right, check_limit) =
        (CheckLeft::value(), CheckRight::value(), CheckLimit::value());
    if check_limit {
        debug_assert_eq!(true, input1.len() + input2.len() >= limit);
        debug_assert_eq!(true, limit <= output.len());
    } else {
        debug_assert_eq!(input1.len() + input2.len(), output.len());
    }
    if check_left && input1.is_empty() {
        output.copy_from_slice(input2);
        return None;
    } else if check_right && input2.is_empty() {
        output.copy_from_slice(input1);
        return None;
    } else {
        unsafe {
            let mut value1 = input1.get_unchecked(i1);
            let mut value2 = input2.get_unchecked(i2);
            for o in output.iter_mut() {
                if check_limit && i_out >= limit {
                    break;
                }
                if value1.lt(value2) {
                    *o = *value1;
                    i1 += 1;
                    i_out += 1;
                    if check_left && i1 >= input1.len() {
                        break;
                    }
                    value1 = input1.get_unchecked(i1);
                } else {
                    *o = *value2;
                    i2 += 1;
                    i_out += 1;
                    if check_right && i2 >= input2.len() {
                        break;
                    }
                    value2 = input2.get_unchecked(i2);
                }
            }
        }
        if check_right && i2 == input2.len() {
            output[(i1 + i2)..].copy_from_slice(&input1[i1..]);
            return None;
        } else if check_left && i1 == input1.len() {
            output[(i1 + i2)..].copy_from_slice(&input2[i2..]);
            return None;
        }
    }
    Some((i1, i2, i_out))
}

/// We can now fuse contiguous slices together back into one.
fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.offset(s1.len() as isize) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

/// Adaptive merge input
struct MergeBlock<'a, T: 'a + Sync + Send> {
    left: &'a [T],
    right: &'a [T],
    output: &'a mut [T],
}

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

impl<'a, T: 'a + Ord + Copy + Sync + Send> Divisible for MergeBlock<'a, T> {
    fn len(&self) -> usize {
        self.output.len()
    }
    fn split(self) -> (Self, Self) {
        let ((left1, left2, left3), (right1, right2, right3)) =
            if self.left.len() > self.right.len() {
                merge_split(self.left, self.right)
            } else {
                let (split_right, split_left) = merge_split(self.right, self.left);
                (split_left, split_right)
            };
        let (output1, output_end) = self.output.split_at_mut(left1.len() + right1.len());
        let (output2, output3) = output_end.split_at_mut(left2.len() + right2.len());
        let (output2a, output2b) = output2.split_at_mut(left2.len());
        output2a.copy_from_slice(left2);
        output2b.copy_from_slice(right2);
        (
            MergeBlock {
                left: left1,
                right: right1,
                output: output1,
            },
            MergeBlock {
                left: left3,
                right: right3,
                output: output3,
            },
        )
    }
}

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>, // we have 2 input slices and one output slice
    i: usize,            // index of slice containing the data
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
            SortingSlices { s: v.0, i: self.i },
            SortingSlices { s: v.1, i: self.i },
        )
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Divisible for SortingSlices<'a, T> {
    fn len(&self) -> usize {
        self.s[0].len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.s[0].len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: 'a + Ord + Copy + Sync + Send> Mergeable for SortingSlices<'a, T> {
    fn fuse(self, other: Self) -> Self {
        let mut slices = self;
        let mut other = other;
        let destination = (0..3usize)
            .filter(|&i| i != slices.i && i != other.i)
            .next()
            .unwrap();
        {
            let i1 = slices.i;
            let (s1, d1) = slices.mut_couple(i1, destination);
            let i2 = other.i;
            let (s2, d2) = other.mut_couple(i2, destination);
            let output = fuse_slices(d1, d2);

            let input = MergeBlock {
                left: s1,
                right: s2,
                output,
            };
            input.work(
                |d, limit| {
                    let remaining = rayon_logs::sequential_task(3, limit, || {
                        partial_manual_merge::<True, True, True, _>(
                            d.left, d.right, d.output, limit,
                        )
                    });
                    (
                        if let Some((i1, i2, io)) = remaining {
                            Some(MergeBlock {
                                left: &d.left[i1..],
                                right: &d.right[i2..],
                                output: &mut d.output[io..],
                            })
                        } else {
                            None
                        },
                        (),
                    )
                },
                Policy::Adaptive(4000),
            )
        }
        SortingSlices {
            s: slices
                .s
                .into_iter()
                .zip(other.s.into_iter())
                .map(|(s1, s2)| fuse_slices(s1, s2))
                .collect(),
            i: destination,
        }
    }
}

//TODO: what is going on with all required sync and send here ??
fn generic_sort<T: Ord + Copy + Sync + Send>(v: &mut [T], policy: Policy) {
    let mut buffer1 = Vec::with_capacity(v.len());
    unsafe { buffer1.set_len(v.len()) }
    let mut buffer2 = Vec::with_capacity(v.len());
    unsafe { buffer2.set_len(v.len()) }
    let vectors = SortingSlices {
        s: vec![v, &mut buffer1, &mut buffer2],
        i: 0,
    };
    let mut result: SortingSlices<T> = vectors.work(
        |d, limit| {
            if d.s[0].len() == limit {
                let mut slice = d;
                rayon_logs::sequential_task(4, limit, || slice.s[slice.i].sort());
                (None, slice)
            } else {
                let (mut start, remaining) = d.split_at(limit);
                rayon_logs::sequential_task(2, limit, || start.s[start.i].sort());
                (Some(remaining), start)
            }
        },
        policy,
    );

    // we might get one extra copy at the end if data is not in the right buffer
    if result.i != 0 {
        let (final_slice, added_buffers) = result.s.split_first_mut().unwrap();
        final_slice.copy_from_slice(added_buffers[result.i - 1]);
    }
}

fn main() {
    let mut v: Vec<u32> = (0..100_000).collect();
    let answer = v.clone();
    let mut ra = ChaChaRng::new_unseeded();
    ra.shuffle(&mut v);

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    //let log = pool.install(|| generic_sort(&mut v, Policy::Adaptive(2000)))
    //    .1;
    //assert_eq!(v, answer);
    //log.save_svg("adapt.svg").expect("failed saving svg");
    pool.compare(
        "join",
        "adaptive",
        || {
            let mut w = v.clone();
            generic_sort(&mut w, Policy::Adaptive(1000))
        },
        || {
            let mut w = v.clone();
            generic_sort(&mut w, Policy::Adaptive(2000))
        },
        "joins_battle.html",
    ).expect("saving logs failed");
}
