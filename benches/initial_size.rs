#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon_adaptive::adaptive_sort;

use criterion::{Criterion, ParameterizedBenchmark};

fn blocks_sizes(c: &mut Criterion) {
    let sizes = vec![100, 300, 500, 1_000, 2_000, 4_000, 8_000, 12_000];
    c.bench(
        "initial size",
        ParameterizedBenchmark::new(
            "adaptive",
            |b, block_size| {
                b.iter_with_setup(
                    || (0..100_000).map(|_| rand::random()).collect::<Vec<u32>>(),
                    |mut v| {
                        adaptive_sort(&mut v, *block_size);
                    },
                )
            },
            sizes,
        ),
    );
}

criterion_group!(benches, blocks_sizes);
criterion_main!(benches);
