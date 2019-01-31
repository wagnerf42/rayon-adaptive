#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon_adaptive::fuse_slices;
use rayon_adaptive::prelude::*;

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
        pool.scope(|s| {
            input_vector
                .as_mut_slice()
                .partial_fold(
                    //TODO: let's also have an auto-dividing fold
                    || None,
                    |possible_previous_slice: Option<&mut [f64]>, input, limit| {
                        let last_elem_prev_slice = possible_previous_slice
                            .as_ref()
                            .and_then(|c| c.last().cloned())
                            .unwrap_or(1.0);
                        let (todo, remaining) = input.divide_at(limit);
                        todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                            *e *= c;
                            *e
                        });
                        (
                            possible_previous_slice
                                .map(|previous| fuse_slices(previous, todo))
                                .or_else(move || Some(todo)),
                            remaining,
                        )
                    },
                )
                .helping_partial_fold(
                    1.0,
                    //TODO: have a nicer fold api
                    |last_elem_prev_slice, remaining_slice, limit| {
                        let (todo, remaining) = remaining_slice.divide_at(limit);
                        (
                            todo.iter_mut().fold(last_elem_prev_slice, |c, e| {
                                *e *= c;
                                *e
                            }),
                            remaining,
                        )
                    },
                    |last_num, dirty_slice| {
                        if let Some(retrieved_slice) = dirty_slice {
                            let last_slice_num = retrieved_slice.last().cloned().unwrap();
                            s.spawn(move |_| {
                                retrieved_slice
                                    .into_adapt_iter()
                                    .for_each(|e| *e *= last_num)
                            });
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
