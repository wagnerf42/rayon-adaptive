#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{merge_sort_itertools, merge_sort_raw};
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

fn merge_sort_benchmarks(c: &mut Criterion) {
    let sizes: Vec<u32> = vec![100_000, 1_000_000, 10_000_000, 50_000_000, 100_000_000];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "sequential",
            |b, input_size| {
                b.iter_with_setup(
                    || {
                        (0..*input_size)
                            .map(|_| rand::random())
                            .collect::<Vec<u32>>()
                    },
                    |mut v| {
                        v.sort();
                        v
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("adaptive sort", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(16)
                        .build()
                        .expect("Thread binder didn't work!");
                    (
                        thread_pool,
                        (0..*input_size)
                            .map(|_| rand::random())
                            .collect::<Vec<u32>>(),
                    )
                },
                |(tp, mut v)| {
                    tp.install(|| {
                        merge_sort_raw(&mut v, *input_size as usize / 16);
                    });
                    v
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(16)
                        .build()
                        .expect("Thread binder didn't work!");
                    (
                        thread_pool,
                        (0..*input_size)
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

    c.bench(
        "merge sort (reversed input)",
        ParameterizedBenchmark::new(
            "sequential",
            |b, input_size| {
                b.iter_with_setup(
                    || (0u32..*input_size).rev().collect::<Vec<u32>>(),
                    |mut v| {
                        v.sort();
                        v
                    },
                )
            },
            sizes,
        )
        .with_function("adaptive", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(16)
                        .build()
                        .expect("Thread binder didn't work!");
                    (thread_pool, (0u32..*input_size).rev().collect::<Vec<u32>>())
                },
                |(tp, mut v)| {
                    tp.install(|| {
                        merge_sort_raw(&mut v, *input_size as usize / 16);
                    });
                    v
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(16)
                        .build()
                        .expect("Thread binder didn't work!");
                    (thread_pool, (0..*input_size).rev().collect::<Vec<u32>>())
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
            config = Criterion::default().sample_size(10).nresamples(10);
                targets = merge_sort_benchmarks
}
criterion_main!(benches);
