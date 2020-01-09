use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::{merge_sort_adaptive_jp, merge_sort_adaptive_rayon};
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

fn main() {
    let mut input = (1..100_001u32).rev().collect::<Vec<u32>>();
    input.shuffle(&mut thread_rng());
    //println!("before {:?}", input);
    #[cfg(feature = "logs")]
    {
        //let p = ThreadPoolBuilder::new()
        //    .num_threads(8)
        //    .build()
        //    .expect("builder failed");
        //let log = p
        //    .logging_install(|| merge_sort_adaptive_rayon(&mut input))
        //    .1;
        //log.save_svg("rayon_sort_log.svg")
        let p = ThreadPoolBuilder::new()
            .num_threads(2)
            .build()
            .expect("builder failed");
        let log = p.logging_install(|| merge_sort_adaptive_jp(&mut input)).1;
        log.save_svg("join_policy_sort_log.svg")
            .expect("saving svg file failed");
    }

    #[cfg(not(feature = "logs"))]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .expect("pool build failed");
        merge_sort_adaptive_rayon(&mut input);
    }
    //println!("after {:?}", input);
    assert_eq!(input, (0..100_001u32).collect::<Vec<u32>>());
}
