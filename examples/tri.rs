#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rand::random;
use rayon::prelude::*;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::adaptive_sort;
use std::iter::repeat_with;

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("building pool failed");

    let mut v: Vec<u32> = repeat_with(random).take(1048576).collect();
    let mut sorted_v = v.clone();
    sorted_v.sort();

    #[cfg(feature = "logs")]
    {
        pool.compare()
            .attach_algorithm("join_context+adapt", || {
                let mut v2 = v.clone();
                adaptive_sort(&mut v2);
                assert_eq!(v2, sorted_v)
            })
            .attach_algorithm("rayon", || {
                let mut v2 = v.clone();
                v2.par_sort();
                assert_eq!(v2, sorted_v)
            })
            .generate_logs("sorts.html")
            .expect("erreur sauvegarde des logs");
    }
}
