extern crate rand;
#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rand::random;
use rayon::ThreadPoolBuilder;

fn main() {
    let v: Vec<u32> = (0..4_000).map(|_| random::<u32>() % 10).collect();
    //    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    #[cfg(feature = "logs")]
    {
        let (filtered, log) = pool.install(|| {
            v.into_adapt_iter()
                .filter(|&i| *i % 2 == 0)
                .cloned()
                .collect::<Vec<u32>>()
        });
        assert_eq!(filtered, answer);
        log.save_svg("filter.svg").expect("failed saving svg");
    }
    #[cfg(not(feature = "logs"))]
    {
        let filtered: Vec<_> = pool.install(|| {
            v.into_adapt_iter()
                .filter(|&i| *i % 2 == 0)
                .map(|&i| i + 1)
                .collect()
        });
        //assert_eq!(filtered, answer);
        let my_vec: Vec<usize> = (0..10000).into_adapt_iter().collect::<Vec<usize>>();
    }
}
