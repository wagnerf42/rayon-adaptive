#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::merge_sort_adaptive;
use std::time::Duration;

use criterion::{Criterion, ParameterizedBenchmark};
const PROBLEM_SIZE: u32 = 100_000_000;

fn merge_sort_benchmarks(c: &mut Criterion) {
    let num_threads: Vec<usize> = vec![
        1, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32, 34, 36, 38, 40, 42, 44, 46,
        48, 50, 52, 54, 56, 58, 60, 62, 64,
    ];
    c.bench_function("sequential sort (random input)", |b| {
        b.iter_with_setup(
            || {
                let thread_pool = ThreadPoolBuilder::new()
                    .num_threads(1)
                    .build()
                    .expect("Thread binder didn't work!");
                (
                    thread_pool,
                    (0..PROBLEM_SIZE)
                        .map(|_| rand::random())
                        .collect::<Vec<u32>>(),
                )
            },
            |(tp, mut v)| {
                tp.install(|| {
                    v.sort();
                });
                v
            },
        )
    });

    c.bench(
        "parallel sorts (random input)",
        ParameterizedBenchmark::new(
            "adaptive sort",
            |b, &nt| {
                b.iter_with_setup(
                    || {
                        let thread_pool = ThreadPoolBuilder::new()
                            .num_threads(nt)
                            .build()
                            .expect("Thread binder didn't work!");
                        (
                            thread_pool,
                            (0..PROBLEM_SIZE)
                                .map(|_| rand::random())
                                .collect::<Vec<u32>>(),
                        )
                    },
                    |(tp, mut v)| {
                        tp.install(|| {
                            merge_sort_adaptive(&mut v);
                        });
                        v
                    },
                )
            },
            num_threads.clone(),
        )
        .with_function("rayon", |b, &nt| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(nt)
                        .build()
                        .expect("Thread binder didn't work!");
                    (
                        thread_pool,
                        (0..PROBLEM_SIZE)
                            .map(|_| rand::random())
                            .collect::<Vec<u32>>(),
                    )
                },
                |(tp, mut v)| {
                    tp.install(|| {
                        v.par_sort();
                    });
                    v
                },
            )
        }),
    );
}

criterion_group! {
    name = benches;
            config = Criterion::default().sample_size(10).warm_up_time(Duration::from_millis(50)).nresamples(200);
                targets = merge_sort_benchmarks
}
criterion_main!(benches);
