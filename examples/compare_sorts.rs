use rand::prelude::*;
use rand::{thread_rng, Rng};
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

fn main() {
    #[cfg(feature = "logs")]
    {
        let p = ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .expect("builder failed");
        p.compare()
            .attach_algorithm_with_setup(
                "sorting with rayon policy only",
                || {
                    let mut input = (1..100_001u32).rev().collect::<Vec<u32>>();
                    input.shuffle(&mut thread_rng());
                    input
                },
                |mut unsorted_vector| {
                    merge_sort_adaptive_rayon(&mut unsorted_vector);
                    unsorted_vector
                },
            )
            .attach_algorithm_with_setup(
                "sorting with a fixed join policy",
                || {
                    let mut input = (1..100_001u32).rev().collect::<Vec<u32>>();
                    input.shuffle(&mut thread_rng());
                    input
                },
                |mut unsorted_vector| {
                    merge_sort_adaptive_jp(&mut unsorted_vector);
                    unsorted_vector
                },
            )
            .generate_logs("sort_comparison.html")
            .expect("logging failed");
    }
}
