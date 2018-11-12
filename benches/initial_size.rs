#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rayon_adaptive::{Policy, vec_gen, solver_adaptive};

use criterion::{Criterion, ParameterizedBenchmark};

fn blocks_sizes(c: &mut Criterion) {
    let sizes = vec![2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
    c.bench(
        "initial size",
        ParameterizedBenchmark::new(
            "adaptive",
            |b, block_size| {
                b.iter_with_setup(
                    || vec_gen(100_000),
                    |v| {
                        solver_adaptive(&v, Policy::Adaptive(*block_size));
                    },
                )
            },
            sizes,
        ),
    );
}

criterion_group!(benches, blocks_sizes);
criterion_main!(benches);
