use rand::prelude::*;
use rand::thread_rng;
use rayon_adaptive::merge_sort_adaptive;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

const PROBLEM_SIZE: u32 = 20_000_000u32;
const NUM_THREADS: usize = 4;

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
        let p = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("builder failed");
        let log = p.logging_install(|| merge_sort_adaptive(&mut input1)).1;
        log.save_svg("fully_parallel_sort.svg")
            .expect("saving svg file failed");
        assert_eq!(input1, solution1);
    }

    #[cfg(not(feature = "logs"))]
    {
        rayon::ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build_global()
            .expect("pool build failed");
        merge_sort_adaptive(&mut input1);
        merge_sort_adaptive(&mut input2);
        assert_eq!(input1, solution1);
        assert_eq!(input2, solution2);
    }
    //println!("after {:?}", input);
}
