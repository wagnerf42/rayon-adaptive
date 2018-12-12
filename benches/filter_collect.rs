#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon_adaptive::prelude::*;

use criterion::Criterion;

fn filter_collect_adaptive(c: &mut Criterion) {
    c.bench_function("adaptive filter_collect(size=10_000_000)", move |b| {
        b.iter(|| {
            (0..10_000_000)
                .into_adapt_iter()
                .filter(|&x| x * 2 == 0)
                .collect::<Vec<usize>>()
        })
    });
    c.bench_function("sequential filter_collect(size=10_000_000)", move |b| {
        b.iter(|| {
            (0..10_000_000)
                .filter(|&x| x * 2 == 0)
                .collect::<Vec<usize>>()
        })
    });
}

criterion_group!(benches, filter_collect_adaptive);
criterion_main!(benches);
