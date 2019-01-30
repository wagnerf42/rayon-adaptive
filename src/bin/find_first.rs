#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;

fn main() {
    let v: Vec<u32> = (0..10_000_000).collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("pool creation failed");
    let answer = pool.install(|| v.into_adapt_iter().find_first(|&e| *e == 4_800_000));
    assert_eq!(answer.cloned().unwrap(), 4_800_000);
}
