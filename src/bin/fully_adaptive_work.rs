#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::scope;
use rayon_adaptive::prelude::*;
use rayon_adaptive::BlockedPower;
use std::iter::repeat;

const SIZE: usize = 1_000_000;

struct PrefixSlice<'a, T: 'a + Send + Sync> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T: 'a + Send + Sync> Divisible for PrefixSlice<'a, T> {
    type Power = BlockedPower;
    fn base_length(&self) -> usize {
        self.slice.len() - self.index
    }
    fn divide(self) -> (Self, Self) {
        let middle = self.base_length() / 2;
        let (left, right) = self.slice.split_at_mut(self.index + middle);
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

impl<'a, T: 'a + Send + Sync> DivisibleIntoBlocks for PrefixSlice<'a, T> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at_mut(self.index + index);
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

fn main() {
    #[cfg(feature = "logs")]
    {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .expect("Thread pool build failed");
        pool.compare()
            .runs_number(1)
            .attach_algorithm_with_setup(
                "adaptive",
                || vec![1.0; SIZE],
                |mut input_vector| {
                    let length = input_vector.len();
                    let input = PrefixSlice {
                        slice: input_vector.as_mut_slice(),
                        index: 0,
                    };

                    {
                        scope(|s| {
                            input
                                .by_blocks(repeat(length / 10))
                                .work(|mut prefix_slice, limit| {
                                    let previous_value = if prefix_slice.index == 0 {
                                        1.0
                                    } else {
                                        prefix_slice
                                            .slice
                                            .get(prefix_slice.index - 1)
                                            .cloned()
                                            .unwrap()
                                    };
                                    prefix_slice.slice
                                        [prefix_slice.index..(prefix_slice.index + limit)]
                                        .iter_mut()
                                        .fold(previous_value, |previous_value, e| {
                                            *e *= previous_value;
                                            *e
                                        });
                                    prefix_slice.index += limit;
                                    prefix_slice
                                })
                                .map(|s| s.slice)
                                .helping_cutting_fold(
                                    1.0,
                                    |last_elem_prev_slice, prefix_slice| {
                                        prefix_slice.slice.iter_mut().fold(
                                            last_elem_prev_slice,
                                            |c, e| {
                                                *e *= c;
                                                *e
                                            },
                                        )
                                    },
                                    |last_num, slice| {
                                        if let Some(last_slice_num) = slice.last().cloned() {
                                            s.spawn(move |_| {
                                                slice.into_adapt_iter().for_each(|e| *e *= last_num)
                                            });
                                            last_num * last_slice_num
                                        } else {
                                            last_num
                                        }
                                    },
                                )
                        });
                    }
                    input_vector
                },
            )
            .generate_logs("prefix.html")
            .expect("failed saving logs");
    }
    #[cfg(not(feature = "logs"))]
    (1..=4).for_each(|number_of_threads| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build()
            .expect("Thread pool build failed");
        let mut input_vector = vec![1.0; SIZE];
        let expected_result: Vec<_> = vec![1.0; SIZE];
        let start = time::precise_time_ns();
        let length = input_vector.len();
        let input = PrefixSlice {
            slice: input_vector.as_mut_slice(),
            index: 0,
        };

        pool.scope(|s| {
            input
                .by_blocks(repeat(length / 10))
                .work(|mut prefix_slice, limit| {
                    let previous_value = if prefix_slice.index == 0 {
                        1.0
                    } else {
                        prefix_slice
                            .slice
                            .get(prefix_slice.index - 1)
                            .cloned()
                            .unwrap()
                    };
                    prefix_slice.slice[prefix_slice.index..(prefix_slice.index + limit)]
                        .iter_mut()
                        .fold(previous_value, |previous_value, e| {
                            *e *= previous_value;
                            *e
                        });
                    prefix_slice.index += limit;
                    prefix_slice
                })
                .map(|s| s.slice)
                .helping_cutting_fold(
                    1.0,
                    |last_elem_prev_slice, prefix_slice| {
                        prefix_slice
                            .slice
                            .iter_mut()
                            .fold(last_elem_prev_slice, |c, e| {
                                *e *= c;
                                *e
                            })
                    },
                    |last_num, slice| {
                        if let Some(last_slice_num) = slice.last().cloned() {
                            s.spawn(move |_| slice.into_adapt_iter().for_each(|e| *e *= last_num));
                            last_num * last_slice_num
                        } else {
                            last_num
                        }
                    },
                )
        });

        let end = time::precise_time_ns();
        let time_taken_ms = ((end - start) as f64) / (1e6 as f64);
        assert_eq!(input_vector, expected_result);

        println!("{}, {}", time_taken_ms, number_of_threads);
    });

    let mut input_vector = vec![1.0; SIZE];
    let expected_result: Vec<_> = vec![1.0; SIZE];
    let start = time::precise_time_ns();
    input_vector.iter_mut().fold(1.0, |value, e| {
        *e *= value;
        *e
    });
    let end = time::precise_time_ns();
    assert_eq!(input_vector, expected_result);
    println!("sequential time: {}", (end - start) as f64 / (1e6 as f64));
}
