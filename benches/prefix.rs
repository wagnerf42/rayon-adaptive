#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::adaptive_prefix;
use std::collections::LinkedList;

use criterion::{Criterion, ParameterizedBenchmark};

fn prefix_adaptive(c: &mut Criterion) {
    let sizes = vec![10_000, 20_000, 50_000, 75_000, 100_000];
    c.bench(
        "prefix (multiplying floats)",
        ParameterizedBenchmark::new(
            "adaptive",
            |b, input_size| {
                b.iter_with_setup(
                    || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                    |mut v| {
                        adaptive_prefix(&mut v, |a, b| *a * *b);
                    },
                )
            },
            sizes,
        ).with_function("sequential", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                |mut v| {
                    v.iter_mut().fold(1.0, |acc, x| {
                        *x *= acc;
                        *x
                    });
                },
            )
        }).with_function("rayon", |b, input_size| {
            b.iter_with_setup(
                || (0..*input_size).map(|_| 1.0).collect::<Vec<f64>>(),
                |mut v| {
                    let blocks = v
                        .par_iter_mut()
                        .fold(
                            || (0.0, 0),
                            |(previous_value, count), x| {
                                *x *= previous_value;
                                (*x, count + 1)
                            },
                        ).map(|(_, count)| {
                            let mut l = LinkedList::new();
                            l.push_back(count);
                            l
                        }).reduce(LinkedList::new, |mut l1, l2| {
                            l1.extend(l2);
                            l1
                        });
                    let mut sizes = blocks.into_iter();
                    let start_size = sizes.next().unwrap();
                    let mut current_position = start_size;
                    for size in sizes {
                        let previous_value = v[current_position - 1];
                        let next_position = current_position + size;
                        v[current_position..next_position]
                            .par_iter_mut()
                            .for_each(|x| {
                                *x *= previous_value;
                            });
                        current_position = next_position;
                    }
                },
            )
        }),
    );
}

criterion_group!(benches, prefix_adaptive);
criterion_main!(benches);
