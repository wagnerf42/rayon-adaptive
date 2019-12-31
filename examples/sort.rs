use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::merge_sort_adaptive;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

const PROBLEM_SIZE: u32 = 1_000_000u32;

fn main() {
    let mut input = (1..PROBLEM_SIZE).rev().collect::<Vec<u32>>();
    input.shuffle(&mut thread_rng());
    let solution = (1..PROBLEM_SIZE).collect::<Vec<u32>>();
    //println!("before {:?}", input);
    #[cfg(feature = "logs")]
    {
        let p = ThreadPoolBuilder::new().build().expect("builder failed");
        let log = p.logging_install(|| merge_sort_adaptive(&mut input)).1;
        log.save_svg("our_log.svg").expect("saving svg file failed");
    }

    #[cfg(not(feature = "logs"))]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(5)
            .build_global()
            .expect("pool build failed");
        merge_sort_adaptive(&mut input);
    }
    //println!("after {:?}", input);
    assert_eq!(input, solution);
}
