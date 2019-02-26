use rand::{thread_rng, Rng};
use rayon::prelude::*;
use rayon_adaptive::{adaptive_sort, adaptive_sort_raw};
const NUM_THREADS: usize = 8;
const SIZE: u32 = 200_000;

fn main() {
    let v: Vec<u32> = (0..SIZE).collect();
    let mut rng = thread_rng();
    let mut random_v_0: Vec<u32> = (0..SIZE).collect();
    let mut random_v_1: Vec<u32> = (0..SIZE).collect();
    let mut random_v_2: Vec<u32> = (0..SIZE).collect();
    rng.shuffle(&mut random_v_0);
    rng.shuffle(&mut random_v_1);
    rng.shuffle(&mut random_v_2);
    //let mut inverted_v: Vec<u32> = (0..SIZE).rev().collect();
    #[cfg(not(feature = "logs"))]
    {
        adaptive_sort(&mut random_v_0);
        adaptive_sort_raw(&mut random_v_1);
        random_v_2.par_sort();
        assert_eq!(v, random_v_0);
        assert_eq!(v, random_v_1);
        assert_eq!(v, random_v_2);
    }
    #[cfg(feature = "logs")]
    {
        let pool = rayon_logs::ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("build pool failed");
        pool.compare()
            .attach_algorithm_with_setup(
                "adaptive sort depjoin",
                || {
                    let mut random_v_0: Vec<u32> = (0..SIZE).collect();
                    rng.shuffle(&mut random_v_0);
                    random_v_0
                },
                |mut vec| {
                    adaptive_sort(&mut vec);
                },
            )
            .attach_algorithm_with_setup(
                "raw sort depjoin",
                || {
                    let mut random_v_1: Vec<u32> = (0..SIZE).collect();
                    rng.shuffle(&mut random_v_1);
                    random_v_1
                },
                |mut vec| {
                    adaptive_sort_raw(&mut vec);
                },
            )
            .attach_algorithm_with_setup(
                "rayon sort",
                || {
                    let mut random_v_2: Vec<u32> = (0..SIZE).collect();
                    rng.shuffle(&mut random_v_2);
                    random_v_2
                },
                |mut vec| {
                    vec.par_sort();
                },
            )
            .generate_logs(format!("sort_comparisons_size_{}.html", SIZE));
    }
    //adaptive_sort(&mut inverted_v);
    //assert_eq!(v, inverted_v);
}
