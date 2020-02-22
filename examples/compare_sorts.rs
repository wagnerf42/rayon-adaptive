use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::{merge_sort_itertools, merge_sort_peek, merge_sort_raw};
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

const PROBLEM_SIZE: u32 = 10_000_000;
const NUM_THREADS: usize = 4;
fn main() {
    #[cfg(feature = "logs")]
    {
        let p = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("builder failed");
        p.compare()
            .attach_algorithm_with_setup(
                "raw sort",
                || {
                    let mut input = (0..PROBLEM_SIZE).collect::<Vec<u32>>();
                    input.shuffle(&mut thread_rng());
                    input
                },
                |mut unsorted_vector| {
                    merge_sort_raw(&mut unsorted_vector, PROBLEM_SIZE as usize / NUM_THREADS);
                    unsorted_vector
                },
            )
            .attach_algorithm_with_setup(
                "peek sort",
                || {
                    let mut input = (0..PROBLEM_SIZE).collect::<Vec<u32>>();
                    input.shuffle(&mut thread_rng());
                    input
                },
                |mut unsorted_vector| {
                    merge_sort_peek(&mut unsorted_vector, PROBLEM_SIZE as usize / NUM_THREADS);
                    unsorted_vector
                },
            )
            .attach_algorithm_with_setup(
                "itertools sort",
                || {
                    let mut input = (0..PROBLEM_SIZE).collect::<Vec<u32>>();
                    input.shuffle(&mut thread_rng());
                    input
                },
                |mut unsorted_vector| {
                    merge_sort_itertools(&mut unsorted_vector, PROBLEM_SIZE as usize / NUM_THREADS);
                    unsorted_vector
                },
            )
            .generate_logs("sort_comparison.html")
            .expect("logging failed");
    }
}
