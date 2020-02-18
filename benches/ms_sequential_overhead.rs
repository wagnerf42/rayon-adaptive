#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::merge_sort_adaptive_jp;
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

fn merge_sort_overhead(c: &mut Criterion) {
    let sizes = vec![100_000, 1_000_000, 10_000_000, 50_000_000, 100_000_000];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "sequential",
            |b, input_size| {
                b.iter_with_setup(
                    || {
                        let thread_pool = ThreadPoolBuilder::new()
                            .num_threads(1)
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
                            v.sort();
                        });
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
                        .num_threads(1)
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
                        merge_sort_adaptive_jp(&mut v, *input_size / 8);
                    });
                    v
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(1)
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
}

criterion_group! {
        name = benches;
            config = Criterion::default().sample_size(10).nresamples(1000);
                targets = merge_sort_overhead
}
criterion_main!(benches);
