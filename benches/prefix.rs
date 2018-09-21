#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{adaptive_prefix, Policy};
use std::collections::LinkedList;

use criterion::Criterion;

fn prefix_adaptive(c: &mut Criterion) {
    c.bench_function("adaptive prefix (size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).map(|_| 1.0).collect::<Vec<f64>>(),
            |mut v| {
                adaptive_prefix(&mut v, |a, b| *a * *b, Policy::Adaptive(10_000));
            },
        )
    });
    c.bench_function("sequential prefix (size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).map(|_| 1.0).collect::<Vec<f64>>(),
            |mut v| {
                v.iter_mut().fold(1.0, |acc, x| {
                    *x *= acc;
                    *x
                });
            },
        )
    });

    c.bench_function("rayon prefix (size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).map(|_| 1.0).collect::<Vec<f64>>(),
            |mut v| {
                let blocks = v
                    .par_iter_mut()
                    .fold(
                        || (0.0, 0),
                        |(previous_value, count), x| {
                            *x *= previous_value;
                            (*x, count + 1)
                        },
                    )
                    .map(|(_, count)| {
                        let mut l = LinkedList::new();
                        l.push_back(count);
                        l
                    })
                    .reduce(LinkedList::new, |mut l1, l2| {
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
    });
}

criterion_group!(benches, prefix_adaptive);
criterion_main!(benches);
