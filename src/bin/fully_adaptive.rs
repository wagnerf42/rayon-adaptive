#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon_adaptive::fuse_slices;
use rayon_adaptive::prelude::*;
use std::iter::repeat;

const SIZE: usize = 1_000_000;

fn main() {
    (1..=4).for_each(|number_of_threads| {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(number_of_threads)
            .build()
            .expect("Thread pool build failed");
        let mut input_vector = vec![1.0; SIZE];
        let expected_result: Vec<_> = vec![1.0; SIZE];
        let start = time::precise_time_ns();
        let length = input_vector.len();
        pool.scope(|s| {
            input_vector
                .as_mut_slice()
                .by_blocks(repeat(length / 10))
                .cutting_fold(
                    || None,
                    |possible_previous_slice: Option<&mut [f64]>, slice| {
                        let last_elem_prev_slice = possible_previous_slice
                            .as_ref()
                            .and_then(|c| c.last().cloned())
                            .unwrap_or(1.0);
                        slice.iter_mut().fold(last_elem_prev_slice, |c, e| {
                            *e *= c;
                            *e
                        });
                        possible_previous_slice
                            .map(|previous| fuse_slices(previous, slice))
                            .or_else(move || Some(slice))
                    },
                )
                .helping_cutting_fold(
                    1.0,
                    |last_elem_prev_slice, slice| {
                        slice.iter_mut().fold(last_elem_prev_slice, |c, e| {
                            *e *= c;
                            *e
                        })
                    },
                    |last_num, dirty_slice| {
                        if let Some(retrieved_slice) = dirty_slice {
                            let last_slice_num = retrieved_slice.last().cloned().unwrap();
                            s.spawn(move |_| {
                                retrieved_slice
                                    .into_adapt_iter()
                                    .for_each(|e| *e *= last_num)
                            });
                            for _ in 0..rayon::current_num_threads() {
                                // we add a protective layer to redirect slaves to steal
                                // each others
                                s.spawn(|_| ());
                            }
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
}
