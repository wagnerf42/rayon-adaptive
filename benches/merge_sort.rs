#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::merge_sort_adaptive;
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

const PROBLEM_SIZE: usize = 100_000_000;
const NUM_THREADS: usize = 16;

fn merge_sort_benchmarks(c: &mut Criterion) {
    let thresholds: Vec<usize> = vec![
        100_000_000,
        50_000_000,
        25_000_000,
        12_500_000,
        6_250_000,
        3_125_000,
        1_562_500,
        781_250,
        390_625,
        195_313,
        97_657,
        48_828,
    ];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "adaptive sort",
            |b, threshold| {
                b.iter_with_setup(
                    || {
                        let thread_pool = ThreadPoolBuilder::new()
                            .num_threads(NUM_THREADS)
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
            thresholds.clone(),
        ),
    );
}

criterion_group! {
        name = benches;
            config = Criterion::default().sample_size(10).nresamples(200);
                targets = merge_sort_benchmarks
}
criterion_main!(benches);
