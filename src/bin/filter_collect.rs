extern crate rand;
#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rand::random;
use rayon::ThreadPoolBuilder;

fn main() {
    let v: Vec<u32> = (0..4_000).map(|_| random::<u32>() % 10).collect();
    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    let filtered: Vec<_> = pool.install(|| {
        v.into_adapt_iter()
            .filter(|&i| *i % 2 == 0)
            .cloned()
            .collect()
    });
    assert_eq!(filtered, answer);
}
