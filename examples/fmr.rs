#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn main() {
    let v: Vec<u32> = (0..2_000_000).collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("building pool failed");

    let even_elements = pool.install(|| {
        let mut vecs = v
            .into_adapt_iter()
            .filter(|&e| *e % 2 == 0)
            .with_policy(Policy::Adaptive(20, 200_000))
            .fold(Vec::new, |mut v, e| {
                v.push(*e);
                v
            })
            .into_iter();
        let final_vec = vecs.next().unwrap();
        vecs.fold(final_vec, |mut f, v| {
            f.extend(v);
            f
        })
    });

    assert_eq!(even_elements.len(), 1_000_000);
}
