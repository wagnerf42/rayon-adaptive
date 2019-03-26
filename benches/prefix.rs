#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{adaptive_prefix, fully_adaptive_prefix};
use std::collections::LinkedList;

use criterion::{Criterion, ParameterizedBenchmark};

fn prefix_adaptive(c: &mut Criterion) {
    let sizes = vec![5_000_000];
    //let sizes = vec![100_000, 1_000_000, 2_000_000];
    c.bench(
        "prefix (adding floats)",
        ParameterizedBenchmark::new(
            "adaptive",
            |b, input_size| {
                b.iter_with_setup(
                    || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                    |mut v| {
                        adaptive_prefix(&mut v, |a, b| *a + *b);
                    },
                )
            },
            sizes,
        )
        .with_function("fully adaptive", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                |mut v| {
                    fully_adaptive_prefix(&mut v, |a, b| *a + *b);
                },
            )
        })
        .with_function("sequential", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                |mut v| {
                    v.iter_mut().fold(0.0, |acc, x| {
                        *x += acc;
                        *x
                    });
                },
            )
        })
        .with_function("rayon with chunks", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                |mut v| {
                    let block_sizes = v.len() / (2 * rayon::current_num_threads());
                    v.par_chunks_mut(block_sizes).for_each(|c| {
                        c.iter_mut().fold(0.0, |last_value, e| {
                            *e += last_value;
                            *e
                        });
                    });
                    let increments: Vec<f64> = v[(block_sizes - 1)..]
                        .iter()
                        .step_by(block_sizes)
                        .scan(0.0, |value, e| {
                            *value += *e;
                            Some(*value)
                        })
                        .collect();
                    v.par_chunks_mut(block_sizes)
                        .skip(1)
                        .zip(increments.par_iter())
                        .for_each(|(c, v)| c.iter_mut().for_each(|e| *e += *v));
                    v
                },
            )
        }),
    );
}

criterion_group!(benches, prefix_adaptive);
criterion_main!(benches);
