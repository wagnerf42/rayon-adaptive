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
    let input: Vec<u32> = (0..1_000_000).rev().collect();
    c.bench_function(
        "adaptive merge sort (size=1_000_000, reversed input)",
        move |b| {
            b.iter(|| {
                let mut v = input.clone();
                adaptive_sort(&mut v);
            })
        },
    );
    c.bench_function(
        "adaptive merge sort (size=1_000_000, random input)",
        move |b| {
            let mut ra = ChaChaRng::new_unseeded();
            b.iter(|| {
                let mut v: Vec<u32> = (0..1_000_000).collect();
                ra.shuffle(&mut v);
                adaptive_sort(&mut v);
            })
        },
    );

    let input: Vec<u32> = (0..1_000_000).rev().collect();
    c.bench_function(
        "rayon merge sort (size=1_000_000, reversed input)",
        move |b| {
            b.iter(|| {
                let mut v = input.clone();
                v.par_sort();
            })
        },
    );
    c.bench_function(
        "rayon merge sort (size=1_000_000, random input)",
        move |b| {
            let mut ra = ChaChaRng::new_unseeded();
            b.iter(|| {
                let mut v: Vec<u32> = (0..1_000_000).collect();
                ra.shuffle(&mut v);
                v.par_sort();
            })
        },
    );
}

criterion_group!(benches, merge_sort_adaptive);
criterion_main!(benches);
