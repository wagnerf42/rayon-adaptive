#[macro_use]
extern crate criterion;
extern crate rayon_adaptive;

use criterion::{Criterion, ParameterizedBenchmark};
use rayon_adaptive::prelude::*;

fn fold_reduce(c: &mut Criterion) {
    let sizes = vec![
        1_000, 10_000, 50_000, 100_000, 1_000_000, 2_000_000, 5_000_000, 10_000_000, 25_000_000,
        50_000_000,
    ];
    c.bench(
        "sum_fmr",
        ParameterizedBenchmark::new(
            "Sequential",
            |b, size| b.iter(|| (0u64..*size).into_iter().sum::<u64>()),
            sizes,
        )
        .with_function("Adaptive", |b, size| {
            b.iter(|| {
                (0u64..*size)
                    .into_par_iter()
                    .fold(|| 0u64, |current_sum, elem| current_sum + elem)
                    .reduce(|| 0u64, |left_sum, right_sum| left_sum + right_sum)
            })
        })
        .with_function("Adaptive interruptible", |b, size| {
            b.iter(|| {
                (0u64..*size)
                    .into_par_iter()
                    .iterator_fold(|some_iter| Some(some_iter.sum::<u64>()))
                    .try_reduce(|| 0, |left_val, right_val| Some(left_val + right_val))
            })
        }),
    );
}

criterion_group!(benches, fold_reduce);
criterion_main!(benches);
