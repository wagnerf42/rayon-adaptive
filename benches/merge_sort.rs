#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::merge_sort_adaptive;
use std::time::Duration;
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

const PROBLEM_SIZE: usize = 100_000_000;

fn merge_sort_benchmarks(c: &mut Criterion) {
    let thresholds: Vec<usize> = vec![
        25_000_000, 6_250_000, 1_562_500, 390_625, 97_657, 48_828, 24_414, 12_207, 6_103,
    ];
    let thread_nums: Vec<usize> = vec![4, 8, 12, 16, 24, 32, 48];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "adaptive sort",
            |b, (num_threads, threshold)| {
                b.iter_with_setup(
                    || {
                        let thread_pool = ThreadPoolBuilder::new()
                            .num_threads(*num_threads)
                            .build()
                            .expect("Thread binder didn't work!");
                        (
                            thread_pool,
                            (
                                (0..PROBLEM_SIZE)
                                    .map(|_| rand::random())
                                    .collect::<Vec<u32>>(),
                                threshold,
                            ),
                        )
                    },
                    |(tp, (mut v, threshold))| {
                        tp.install(|| {
                            merge_sort_adaptive(&mut v, *threshold);
                        });
                        v
                    },
                )
            },
            thread_nums
                .clone()
                .into_iter()
                .flat_map(|nt| thresholds.clone().into_iter().map(move |th| (nt, th))),
        ),
    );
}

criterion_group! {
        name = benches;
            config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(500)).nresamples(200);
                targets = merge_sort_benchmarks
}
criterion_main!(benches);
