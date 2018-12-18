use rand::seq::SliceRandom;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use rayon_logs::ThreadPoolBuilder;

const SIZE: u32 = 200_000;
// this file is expected to be run with the logs feature enabled:
// cargo run --features logs --example max --release
fn main() {
    let mut v: Vec<u32> = (0..SIZE).collect();
    let mut rng = rand::thread_rng();
    v.shuffle(&mut rng);
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    pool.compare()
        .runs_number(500)
        //        .attach_algorithm("truly sequential", || {
        //            assert_eq!(v.iter().max().cloned(), Some(SIZE - 1))
        //        })
        //        .attach_algorithm("sequential", || {
        //            assert_eq!(
        //                v.into_adapt_iter()
        //                    .with_policy(Policy::Sequential)
        //                    .max()
        //                    .cloned(),
        //                Some(SIZE - 1)
        //            )
        //        })
        .attach_algorithm("join (block size=1000)", || {
            assert_eq!(
                v.into_adapt_iter()
                    .with_policy(Policy::Join(1000))
                    .max()
                    .cloned(),
                Some(SIZE - 1)
            )
        })
        //        .attach_algorithm("join (block size=100)", || {
        //            assert_eq!(
        //                v.into_adapt_iter()
        //                    .with_policy(Policy::Join(100))
        //                    .max()
        //                    .cloned(),
        //                Some(SIZE - 1)
        //            )
        //        })
        .attach_algorithm("join-context (block size=10)", || {
            assert_eq!(
                v.into_adapt_iter()
                    .with_policy(Policy::JoinContext(10))
                    .max()
                    .cloned(),
                Some(SIZE - 1)
            )
        })
        .attach_algorithm("adaptive", || {
            assert_eq!(v.into_adapt_iter().max().cloned(), Some(SIZE - 1))
        })
        .generate_logs("comparing_schedulers_on_max.html")
        .expect("failed comparisons");
}
