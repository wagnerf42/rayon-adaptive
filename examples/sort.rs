use rand::prelude::*;
use rand::{thread_rng, Rng};
use rayon_adaptive::merge_sort_adaptive;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

const PROBLEM_SIZE: u32 = 20_000_000u32;

fn main() {
    let mut input1 = (1..PROBLEM_SIZE).collect::<Vec<u32>>();
    input1.shuffle(&mut thread_rng());
    let mut input2 = (1..PROBLEM_SIZE / 5)
        .chain(1..PROBLEM_SIZE / 5)
        .chain(1..PROBLEM_SIZE / 5)
        .chain(1..PROBLEM_SIZE / 5)
        .chain(1..PROBLEM_SIZE / 5)
        .collect::<Vec<u32>>();
    input1.shuffle(&mut thread_rng());
    input2.shuffle(&mut thread_rng());
    let solution1 = (1..PROBLEM_SIZE).collect::<Vec<u32>>();
    let solution2 = (1..PROBLEM_SIZE / 5)
        .flat_map(|num| std::iter::repeat(num).take(5))
        .collect::<Vec<u32>>();
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
            .num_threads(4)
            .build_global()
            .expect("pool build failed");
        merge_sort_adaptive(&mut input1);
        merge_sort_adaptive(&mut input2);
    }
    //println!("after {:?}", input);
    assert_eq!(input1, solution1);
    assert_eq!(input2, solution2);
}
