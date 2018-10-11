#[macro_use]
extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rayon_adaptive::{filter_collect, Policy};

use criterion::Criterion;

fn filter_collect_adaptive(c: &mut Criterion) {
    c.bench_function("adaptive filter_collect(size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).map(|i| i % 2).collect::<Vec<u32>>(),
            |v| filter_collect(&v, |&e| *e % 2 == 0, Policy::Adaptive(10_000)),
        )
    });
    c.bench_function("sequential filter_collect(size=10_000_000)", move |b| {
        b.iter_with_setup(
            || (0..10_000_000).map(|i| i % 2).collect::<Vec<u32>>(),
            |v| {
                v.iter()
                    .filter(|&e| *e % 2 == 0)
                    .cloned()
                    .collect::<Vec<u32>>()
            },
        )
    });
}

criterion_group!(benches, filter_collect_adaptive);
criterion_main!(benches);
