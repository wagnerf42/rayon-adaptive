#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rand::{ChaChaRng, Rng};
use rayon::prelude::*;
use rayon_adaptive::adaptive_sort;

use criterion::Criterion;

fn merge_sort_adaptive(c: &mut Criterion) {
    c.bench_function(
        "adaptive merge sort (size=1_000_000, reversed input)",
        move |b| {
            b.iter_with_setup(
                || (0..1_000_000).rev().collect::<Vec<u32>>(),
                |mut v| {
                    adaptive_sort(&mut v);
                },
            )
        },
    );
    let mut all_numbers: Vec<u32> = (0..1_000_000).collect();
    let mut ra = ChaChaRng::new_unseeded();
    c.bench_function(
        "adaptive merge sort (size=1_000_000, random input)",
        move |b| {
            b.iter_with_setup(
                || {
                    ra.shuffle(&mut all_numbers);
                    all_numbers.clone()
                },
                |mut v| {
                    adaptive_sort(&mut v);
                },
            )
        },
    );

    c.bench_function(
        "rayon merge sort (size=1_000_000, reversed input)",
        move |b| {
            b.iter_with_setup(
                || (0..1_000_000).rev().collect::<Vec<u32>>(),
                |mut v| {
                    v.par_sort();
                },
            )
        },
    );
    let mut all_numbers: Vec<u32> = (0..1_000_000).collect();
    c.bench_function(
        "rayon merge sort (size=1_000_000, random input)",
        move |b| {
            b.iter_with_setup(
                || {
                    ra.shuffle(&mut all_numbers);
                    all_numbers.clone()
                },
                |mut v| {
                    v.par_sort();
                },
            )
        },
    );
}

criterion_group!(benches, merge_sort_adaptive);
criterion_main!(benches);
