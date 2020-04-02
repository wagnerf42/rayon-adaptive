#[macro_use]
extern crate criterion;
extern crate itertools;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use itertools::Itertools;
use rand::prelude::*;
use rayon_adaptive::prelude::*;

use criterion::{Criterion, ParameterizedBenchmark};

struct SliceIter<'a, T: 'a> {
    slice: &'a [T],
    index: usize,
}

impl<'a, T: 'a> SliceIter<'a, T> {
    fn new(slice: &'a [T]) -> Self {
        SliceIter { slice, index: 0 }
    }
    fn completed(&self) -> bool {
        self.slice.len() <= self.index
    }
    fn peek(&self) -> &T {
        unsafe { self.slice.get_unchecked(self.index) }
    }
}

impl<'a, T: 'a> Iterator for SliceIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.completed() {
            None
        } else {
            let index = self.index;
            self.index += 1;
            Some(unsafe { self.slice.get_unchecked(index) })
        }
    }
}

struct MergeIter<'a, T: 'a> {
    left: SliceIter<'a, T>,
    right: SliceIter<'a, T>,
}

impl<'a, T: 'a> MergeIter<'a, T> {
    fn new(left: SliceIter<'a, T>, right: SliceIter<'a, T>) -> Self {
        MergeIter { left, right }
    }
}

impl<'a, T: 'a + Ord> Iterator for MergeIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.left.completed() {
            self.right.next()
        } else if self.right.completed() {
            self.left.next()
        } else if self.left.peek() <= self.right.peek() {
            self.left.next()
        } else {
            self.right.next()
        }
    }
}

fn safe_manual_merge(left: &[u32], right: &[u32], output: &mut [u32]) {
    let sizes = [left.len(), right.len()];
    let inputs = [left, right];
    let mut indices = [0usize; 2];
    for o in output {
        let direction = if indices[0] >= sizes[0] {
            1
        } else if indices[1] >= sizes[1] {
            0
        } else if inputs[0][indices[0]] >= inputs[1][indices[1]] {
            0
        } else {
            1
        };
        *o = inputs[direction][indices[direction]];
        indices[direction] += 1;
    }
}

fn unsafe_manual_merge(left: &[u32], right: &[u32], output: &mut [u32]) {
    let sizes = [left.len(), right.len()];
    let inputs = [left, right];
    let mut indices = [0usize; 2];
    for o in output {
        let direction = if indices[0] >= sizes[0] {
            1
        } else if indices[1] >= sizes[1] {
            0
        } else if unsafe {
            inputs[0].get_unchecked(indices[0]) >= inputs[1].get_unchecked(indices[1])
        } {
            0
        } else {
            1
        };
        *o = unsafe { *inputs[direction].get_unchecked(indices[direction]) };
        indices[direction] += 1;
    }
}

fn unsafe_manual_merge2(left: &[u32], right: &[u32], output: &mut [u32]) {
    let mut left_index = 0;
    let mut right_index = 0;
    for o in output {
        unsafe {
            if left_index >= left.len() {
                *o = *right.get_unchecked(right_index);
                right_index += 1;
            } else if right_index >= right.len() {
                *o = *left.get_unchecked(left_index);
                left_index += 1;
            } else if left.get_unchecked(left_index) <= right.get_unchecked(right_index) {
                *o = *left.get_unchecked(left_index);
                left_index += 1;
            } else {
                *o = *right.get_unchecked(right_index);
                right_index += 1;
            };
        }
    }
}

fn manual_slice_iter(left: &[u32], right: &[u32], output: &mut [u32]) {
    let mut left_iter = SliceIter::new(left);
    let mut right_iter = SliceIter::new(right);
    for o in output {
        if left_iter.completed() {
            *o = *right_iter.next().unwrap();
        } else if right_iter.completed() {
            *o = *left_iter.next().unwrap();
        } else if left_iter.peek() <= right_iter.peek() {
            *o = *left_iter.next().unwrap(); // TODO: remove unwrap ?
        } else {
            *o = *right_iter.next().unwrap();
        };
    }
}

fn manual_merge_iter(left: &[u32], right: &[u32], output: &mut [u32]) {
    let left_iter = SliceIter::new(left);
    let right_iter = SliceIter::new(right);
    let mut merge_iter = MergeIter::new(left_iter, right_iter);
    output.iter_mut().zip(merge_iter).for_each(|(o, i)| *o = *i);
}

//TODO: this will be very bad if one block ends up being small
// we should fall back to another algorithm in this case
fn unsafe_very_manual_merge(left: &[u32], right: &[u32], mut output: &mut [u32]) {
    let mut left_index = 0;
    let mut right_index = 0;
    loop {
        let remaining_left_size = left.len() - left_index;
        let remaining_right_size = right.len() - right_index;
        let block_size = std::cmp::min(remaining_left_size, remaining_right_size);
        if block_size == 0 {
            break;
        }
        output[..block_size].iter_mut().for_each(|o| unsafe {
            if left.get_unchecked(left_index) <= right.get_unchecked(right_index) {
                *o = *left.get_unchecked(left_index);
                left_index += 1;
            } else {
                *o = *right.get_unchecked(right_index);
                right_index += 1;
            }
        });
        output = &mut output[block_size..];
    }
    if left_index != left.len() {
        output.copy_from_slice(&left[left_index..])
    } else {
        output.copy_from_slice(&right[right_index..])
    }
}

fn interleaved_input(input_size: u32) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let (mut left, mut right): (Vec<_>, Vec<_>) = (0..input_size).tuples().unzip();
    let output = vec![0u32; input_size as usize];
    let mut rng = thread_rng();
    left.shuffle(&mut rng);
    right.shuffle(&mut rng);
    (left, right, output)
}

fn merge_benchmarks(c: &mut Criterion) {
    let sizes: Vec<u32> = vec![100_000, 500_000, 1_000_000, 2_000_000, 5_000_000];
    // let sizes: Vec<u32> = vec![100_000, 1_000_000, 10_000_000, 50_000_000, 100_000_000];
    c.bench(
        "merge (random input, interleaved, shuffled)",
        ParameterizedBenchmark::new(
            "itertool merge",
            |b, input_size| {
                b.iter_with_setup(
                    || interleaved_input(*input_size),
                    |(left, right, mut output)| {
                        left.iter()
                            .merge(right.iter())
                            .zip(output.iter_mut())
                            .for_each(|(i, o)| *o = *i);
                        (left, right, output)
                    },
                )
            },
            sizes.clone(),
        )
//        .with_function("safe manual merge", |b, input_size| {
//            b.iter_with_setup(
//                || interleaved_input(*input_size),
//                |(left, right, mut output)| {
//                    safe_manual_merge(&left, &right, &mut output);
//                    (left, right, output)
//                },
//            )
//        })
        .with_function("manual slice iter", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    manual_slice_iter(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
        .with_function("manual merge iter", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    manual_merge_iter(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
//        .with_function("unsafe manual merge", |b, input_size| {
//            b.iter_with_setup(
//                || interleaved_input(*input_size),
//                |(left, right, mut output)| {
//                    unsafe_manual_merge(&left, &right, &mut output);
//                    (left, right, output)
//                },
//            )
//        })
        .with_function("unsafe manual merge 2", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    unsafe_manual_merge2(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
        .with_function("unsafe very manual merge", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    unsafe_very_manual_merge(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
//        .with_function("adaptive generic merge", |b, input_size| {
//            b.iter_with_setup(
//                || {
//                    let thread_pool = rayon::ThreadPoolBuilder::new()
//                        .num_threads(1)
//                        .build()
//                        .expect("Thread pool creation failed!");
//                    (thread_pool, interleaved_input(*input_size))
//                },
//                |(tp, (left, right, mut output))| {
//                    // TODO: I wonder if the install is not taking time here
//                    tp.install(|| {
//                        left.par_iter()
//                            .merge(right.par_iter())
//                            .directional_zip(output.par_iter_mut())
//                            .for_each(|(i, o)| *o = *i)
//                    });
//                    (left, right, output)
//                },
//            )
//        }),
    );
}

criterion_group! {
    name = benches;
            config = Criterion::default().sample_size(10).nresamples(100);
                targets = merge_benchmarks
}
criterion_main!(benches);
