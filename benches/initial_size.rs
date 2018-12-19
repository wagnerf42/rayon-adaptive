#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use rand::seq::SliceRandom;
use rayon_adaptive::{prelude::*, Policy};

use criterion::{Criterion, ParameterizedBenchmark};
const INPUT_SIZE: u32 = 100_000;

//fn vec_gen(size: u32) -> Vec<u32> {
//    let mut v: Vec<u32> = (0..size).collect();
//    let mut rng = rand::thread_rng();
//    v.shuffle(&mut rng);
//    v
//}

fn blocks_sizes(c: &mut Criterion) {
    let sizes = vec![
        60, 80, 100, 120, 140, 160, 200, 500, 800, 2000, 4000, 6000, 10000,
    ];
    let sizes = sizes
        .into_iter()
        .map(|block_size| INPUT_SIZE / block_size)
        .collect::<Vec<_>>();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool build failed");
    c.bench(
        "initial size",
        ParameterizedBenchmark::new(
            "adaptive",
            move |b, block_size| {
                b.iter(|| {
                    pool.install(|| {
                        assert_eq!(
                            (0..INPUT_SIZE as usize)
                                .into_adapt_iter()
                                .with_policy(Policy::Adaptive(
                                    (INPUT_SIZE / *block_size) as usize,
                                    (INPUT_SIZE / *block_size) as usize,
                                ))
                                .max(),
                            Some((INPUT_SIZE - 1) as usize)
                        );
                    });
                })
            },
            sizes,
        ),
    );
}

criterion_group!(benches, blocks_sizes);
criterion_main!(benches);
