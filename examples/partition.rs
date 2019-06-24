#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn main() {
    let mut input = (1..1_000_000).collect::<Vec<u64>>();

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(3)
        .build()
        .expect("Thread pool build failed");

    let sum_par: u64 = pool.install(|| {
        input
            .into_par_iter()
            .partition(3)
            .with_policy(Policy::Join(400_000))
            .sum()
    });

    println!("sum: {}", sum_par)
}
