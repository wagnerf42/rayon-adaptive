#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::merge_sort_adaptive;
//use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

fn merge_sort_benchmarks(c: &mut Criterion) {
    let sizes = vec![10_000_000];
    //let sizes = vec![50_000, 100_000, 150_000, 262_144, 1_000_000];
    c.bench(
        "merge sort (random input)",
        ParameterizedBenchmark::new(
            "sequential",
            |b, input_size| {
                b.iter_with_setup(
                    || {
                        //        (0..*input_size)
                        //            .map(|_| rand::random())
                        //            .collect::<Vec<u32>>()
                    },
                    |()| {
                        //        v.sort();
                        //        v
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("adaptive sort", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = rayon::ThreadPoolBuilder::new()
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
                        merge_sort_adaptive(&mut v);
                    });
                    v
                },
            )
        })
        .with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = rayon::ThreadPoolBuilder::new()
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

    //c.bench(
    //    "merge sort (reversed input)",
    //    ParameterizedBenchmark::new(
    //        "sequential",
    //        |b, input_size| {
    //            b.iter_with_setup(
    //                || (0..*input_size).rev().collect::<Vec<u32>>(),
    //                |mut v| {
    //                    v.sort();
    //                    v
    //                },
    //            )
    //        },
    //        sizes,
    //    )
    //    .with_function("adaptive", |b, input_size| {
    //        b.iter_with_setup(
    //            || (0..*input_size).rev().collect::<Vec<u32>>(),
    //            |mut v| {
    //                merge_sort_adaptive(&mut v);
    //                v
    //            },
    //        )
    //    })
    //    .with_function("rayon", |b, input_size| {
    //        b.iter_with_setup(
    //            || (0..*input_size).rev().collect::<Vec<u32>>(),
    //            |mut v| {
    //                v.par_sort();
    //                v
    //            },
    //        )
    //    }),
    //);
}

criterion_group!(benches, merge_sort_benchmarks);
criterion_main!(benches);
