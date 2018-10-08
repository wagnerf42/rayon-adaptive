#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon::prelude::*;
use rayon_adaptive::{find_first, Policy};

use criterion::Criterion;

fn find_first_adaptive(c: &mut Criterion) {
    c.bench_function("adaptive find_first(size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).collect::<Vec<u32>>(),
            |v| {
                assert_eq!(
                    find_first(&v, |&e| *e == 4_800_000, Policy::Adaptive(10_000)).unwrap(),
                    4_800_000
                )
            },
        )
    });
    c.bench_function("sequential find_first(size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).collect::<Vec<u32>>(),
            |v| {
                assert_eq!(
                    v.iter().find(|&e| *e == 4_800_000).cloned().unwrap(),
                    4_800_000
                )
            },
        )
    });
    c.bench_function("rayon find_first(size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).collect::<Vec<u32>>(),
            |v| {
                assert_eq!(
                    v.par_iter()
                        .find_first(|&e| *e == 4_800_000)
                        .cloned()
                        .unwrap(),
                    4_800_000
                )
            },
        )
    });
}

criterion_group!(benches, find_first_adaptive);
criterion_main!(benches);
