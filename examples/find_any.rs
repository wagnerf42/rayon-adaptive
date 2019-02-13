#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("building pool failed");
    let r = pool.install(|| {
        (0..100_000)
            .into_adapt_iter()
            .with_policy(Policy::Join(50_000))
            .find_any(|&x| x % 20_000 == 19_999)
    });
    println!("r: {:?}", r);
    assert_eq!(r.unwrap() % 20_000, 19_999);
}
