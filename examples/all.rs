#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("Pool creation failed");
    pool.install(|| {
        assert!(!(0u64..10_000_000)
            .into_par_iter()
            .with_policy(Policy::Adaptive(10_000, 50_000))
            //.by_blocks(repeat(100_000))
            .all(|e| e != 7_800_000))
    })
}
