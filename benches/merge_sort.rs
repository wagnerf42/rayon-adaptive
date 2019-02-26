#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{adaptive_sort, adaptive_sort_raw};

use criterion::{Criterion, ParameterizedBenchmark};

fn merge_sort_adaptive(c: &mut Criterion) {
    let sizes = vec![50_000, 100_000, 150_000, 262_144];
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
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("adaptive", |b, input_size| {
            b.iter_with_setup(
                || {
                    (0..*input_size)
                        .map(|_| rand::random())
                        .collect::<Vec<u32>>()
                },
                |mut v| {
                    adaptive_sort(&mut v);
                },
            )
        })
        .with_function("adaptive raw", |b, input_size| {
            b.iter_with_setup(
                || {
                    (0..*input_size)
                        .map(|_| rand::random())
                        .collect::<Vec<u32>>()
                },
                |mut v| {
                    adaptive_sort_raw(&mut v);
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || {
                    (0..*input_size)
                        .map(|_| rand::random())
                        .collect::<Vec<u32>>()
                },
                |mut v| {
                    v.par_sort();
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
                    || (0..*input_size).rev().collect::<Vec<u32>>(),
                    |mut v| {
                        v.sort();
                    },
                )
            },
            sizes,
        )
        .with_function("adaptive", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).rev().collect::<Vec<u32>>(),
                |mut v| {
                    adaptive_sort(&mut v);
                },
            )
        })
        .with_function("adaptive raw", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).rev().collect::<Vec<u32>>(),
                |mut v| {
                    adaptive_sort_raw(&mut v);
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).rev().collect::<Vec<u32>>(),
                |mut v| {
                    v.par_sort();
                },
            )
        }),
    );
}

criterion_group!(benches, merge_sort_adaptive);
criterion_main!(benches);
