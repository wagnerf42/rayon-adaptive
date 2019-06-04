#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use criterion::{Criterion, ParameterizedBenchmark};
use rand::Rng;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn all_adaptive(c: &mut Criterion) {
    let sizes = vec![5_000_000];
    // let sizes = vec![100_000, 1_000_000, 2_000_000];
    c.bench(
        "all",
        ParameterizedBenchmark::new(
            "adaptive",
            |b, input_size| {
                b.iter_with_setup(
                    || rand::thread_rng().gen_range(0, *input_size),
                    |idx| {
                        (0u64..*input_size)
                            .into_par_iter()
                            .with_policy(Policy::Adaptive(10_000, 50_000))
                            .all(|e| e != idx)
                    },
                )
            },
            sizes,
        )
        .with_function("rayon 1", |b, input_size| {
            b.iter_with_setup(
                || rand::thread_rng().gen_range(0, *input_size),
                |idx| {
                    (0u64..*input_size)
                        .into_par_iter()
                        .with_policy(Policy::Rayon(1))
                        .all(|e| e != idx)
                },
            )
        })
        .with_function("sequential", |b, input_size| {
            b.iter_with_setup(
                || rand::thread_rng().gen_range(0, *input_size),
                |idx| {
                    (0u64..*input_size)
                        .into_par_iter()
                        .with_policy(Policy::Sequential)
                        .all(|e| e != idx)
                },
            )
        }),
    );
}

criterion_group!(benches, all_adaptive);
criterion_main!(benches);
