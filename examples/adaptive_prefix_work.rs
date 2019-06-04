#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon_adaptive::prelude::*;
use rayon_adaptive::IndexedPower;
use std::cmp::min;
use std::iter::repeat;
use std::slice::IterMut;

const SIZE: usize = 10_000_000;

struct PrefixSlice<'a, T: 'a + Send + Sync> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T: 'a + Send + Sync> Divisible for PrefixSlice<'a, T> {
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        Some(self.slice.len() - self.index)
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let checked_index = min(self.index + index, self.slice.len()); // TODO: check sizes inside scheduler
        let (left, right) = self.slice.split_at_mut(checked_index);
        (
            PrefixSlice {
                slice: left,
                index: self.index,
            },
            PrefixSlice {
                slice: right,
                index: 0,
            },
        )
    }
}

impl<'a, T: 'a + Send + Sync> IntoIterator for PrefixSlice<'a, T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.slice[self.index..]).into_iter()
    }
}

fn main() {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("Thread pool build failed");

    let mut input_vector = repeat(1).take(SIZE).collect::<Vec<u64>>();
    let expected_result: Vec<_> = (1..).take(SIZE).collect();
    let start = time::precise_time_ns();
    let length = input_vector.len();
    let input = PrefixSlice {
        slice: input_vector.as_mut_slice(),
        index: 0,
    };
    pool.scope(|s| {
        input
            .with_help_work(|mut prefix_slice, limit| {
                let previous_value = if prefix_slice.index == 0 {
                    0
                } else {
                    prefix_slice
                        .slice
                        .get(prefix_slice.index - 1)
                        .cloned()
                        .unwrap()
                };
                let len = prefix_slice.slice.len();
                let checked_limit = min(len, prefix_slice.index + limit); // TODO: keep check here or move in scheduler
                prefix_slice.slice[prefix_slice.index..checked_limit]
                    .iter_mut()
                    .fold(previous_value, |previous_value, e| {
                        *e += previous_value;
                        *e
                    });
                prefix_slice.index = checked_limit;
                prefix_slice
            })
            .by_blocks(repeat(length / 10))
            .fold(
                0,
                |p, e| {
                    *e += p;
                    *e
                },
                |last_num, prefix_slice| {
                    let slice = prefix_slice.slice;
                    if let Some(last_slice_num) = slice.last().cloned() {
                        s.spawn(move |_| slice.into_par_iter().for_each(|e| *e += last_num));
                        last_num + last_slice_num
                    } else {
                        last_num
                    }
                },
            )
    });

    let end = time::precise_time_ns();
    let time_taken_ms = ((end - start) as f64) / (1e6 as f64);
    assert_eq!(input_vector, expected_result);

    println!("time taken with 2 threads : {}", time_taken_ms);

    // now do a sequential run
    let mut input_vector = repeat(1).take(SIZE).collect::<Vec<u64>>();
    let start = time::precise_time_ns();
    input_vector.iter_mut().fold(0, |c, e| {
        *e += c;
        *e
    });
    let end = time::precise_time_ns();
    let time_taken_ms = ((end - start) as f64) / (1e6 as f64);
    assert_eq!(input_vector, expected_result);
    println!("time taken sequentially: {}", time_taken_ms);
}
