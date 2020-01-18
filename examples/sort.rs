use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::{merge_sort_adaptive_jp, merge_sort_adaptive_rayon};
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

fn main() {
    let mut input = (1..100_000_001u32).rev().collect::<Vec<u32>>();
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
        let thresholds: Vec<usize> = vec![100_000_000, 50_000_000, 25_000_000, 12_500_000, 6_250_000, 3_125_000, 1_562_500, 781_250, 390_625, 195_313, 97_657, 48_828];
        thresholds.into_iter().for_each(|threshold|{
        let p = ThreadPoolBuilder::new()
            .num_threads(16)
            .build()
            .expect("builder failed");
        let log = p.logging_install(|| merge_sort_adaptive_jp(&mut input, threshold)).1;
        log.save_svg(format!("join_policy_sort_log_{}.svg", threshold))
            .expect("saving svg file failed");
        });
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
    assert_eq!(input, (1..100_000_001u32).collect::<Vec<u32>>());
}
