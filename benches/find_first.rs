#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::prelude::*;

use criterion::{Criterion, ParameterizedBenchmark};

fn find_first_adaptive(c: &mut Criterion) {
    let sizes = vec![100_000, 200_000, 400_000, 600_000];
    c.bench(
        "find first random element",
        ParameterizedBenchmark::new(
            "sequential",
            |b, input_size| {
                b.iter_with_setup(
                    || (0..*input_size).collect::<Vec<u32>>(),
                    |v| {
                        let target = rand::random::<u32>() % input_size;
                        assert_eq!(v.iter().find(|&e| *e == target).cloned().unwrap(), target)
                    },
                )
            },
            sizes,
        ).with_function("adaptive", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).collect::<Vec<u32>>(),
                |v| {
                    let target = rand::random::<u32>() % input_size;
                    assert_eq!(
                        v.into_adapt_iter()
                            .find_first(|&e| *e == target)
                            .cloned()
                            .unwrap(),
                        target
                    )
                },
            )
        }).with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).collect::<Vec<u32>>(),
                |v| {
                    let target = rand::random::<u32>() % input_size;
                    assert_eq!(
                        v.par_iter().find_first(|&e| *e == target).cloned().unwrap(),
                        target
                    )
                },
            )
        }),
    );
}

criterion_group!(benches, find_first_adaptive);
criterion_main!(benches);
