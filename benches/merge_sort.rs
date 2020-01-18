#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::merge_sort_adaptive_jp;
use thread_binder::ThreadPoolBuilder;

use criterion::{Criterion, ParameterizedBenchmark};

const PROBLEM_SIZE: usize = 100_000_001;

fn merge_sort_benchmarks(c: &mut Criterion) {
    //let sizes = vec![50_000, 100_000, 150_000, 262_144, 1_000_000];
    let thresholds: Vec<usize> = vec![100_000_000, 50_000_000, 25_000_000, 12_500_000, 6_250_000, 3_125_000, 1_562_500, 781_250, 390_625, 195_313, 97_657, 48_828];
    c.bench(
        "merge sort (random input)",
        //ParameterizedBenchmark::new(
        //    "sequential",
        //    |b, input_size| {
        //        b.iter_with_setup(
        //            || {
        //                (0..*input_size)
        //                    .map(|_| rand::random())
        //                    .collect::<Vec<u32>>()
        //            },
        //            |mut v| {
        //                v.sort();
        //                v
        //            },
        //        )
        //    },
        //    sizes.clone(),
        //)
        
        ParameterizedBenchmark::new(
            "adaptive sort",
            |b, threshold| {
            b.iter_with_setup(
                || {
                    let thread_pool = ThreadPoolBuilder::new()
                        .num_threads(16)
                        .build()
                        .expect("Thread binder didn't work!");
                    (
                        thread_pool,
                        ((0..PROBLEM_SIZE)
                            .map(|_| rand::random())
                            .collect::<Vec<u32>>(), threshold)
                    )
                },
                |(tp, (mut v, threshold))| {
                    tp.install(|| {
                        merge_sort_adaptive_jp(&mut v, *threshold);
                    });
                    v
                },
            )},
            thresholds.clone(),
        )
        //.with_function("rayon", |b, input_size| {
        //    b.iter_with_setup(
        //        || {
        //            let thread_pool = rayon::ThreadPoolBuilder::new()
        //                .num_threads(1)
        //                .build()
        //                .expect("Thread binder didn't work!");
        //            (
        //                thread_pool,
        //                (0..*input_size)
        //                    .map(|_| rand::random())
        //                    .collect::<Vec<u32>>(),
        //            )
        //        },
        //        |(tp, mut v)| {
        //            tp.install(|| {
        //                v.par_sort();
        //            });
        //            v
        //        },
        //    )
        //}),
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

criterion_group!{
        name = benches;
            config = Criterion::default().sample_size(10).nresamples(10);
                targets = merge_sort_benchmarks
}
criterion_main!(benches);
