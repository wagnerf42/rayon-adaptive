#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{merge_sort_itertools, merge_sort_peek, merge_sort_raw};
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

fn merge_policy(c: &mut Criterion) {
    let sizes = vec![100_000, 1_000_000, 10_000_000, 50_000_000, 100_000_000];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "itertools",
            |b, input_size| {
                b.iter_with_setup(
                    || {
                        let thread_pool = ThreadPoolBuilder::new()
                            .num_threads(4)
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
                            merge_sort_itertools(&mut v, *input_size / 2);
                        });
                        v
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("peeking iterator", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(4)
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
                        merge_sort_peek(&mut v, *input_size / 2);
                    });
                    v
                },
            )
        })
        .with_function("raw", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(4)
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
                        merge_sort_raw(&mut v, *input_size / 2);
                    });
                    v
                },
            )
        }),
    );
}

criterion_group! {
        name = benches;
            config = Criterion::default().sample_size(10).nresamples(1000);
                targets = merge_policy
}
criterion_main!(benches);
