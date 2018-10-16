extern crate rand;
#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rand::random;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::filter_collect;

fn main() {
    let v: Vec<u32> = (0..4_000_000).map(|_| random::<u32>() % 10).collect();
    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    #[cfg(feature = "logs")]
    {
        let (filtered, log) = pool.install(|| filter_collect(&v, |&i| *i % 2 == 0));
        assert_eq!(filtered, answer);
        log.save_svg("filter.svg").expect("failed saving svg");
    }
    #[cfg(not(feature = "logs"))]
    {
        let filtered = pool.install(|| filter_collect(&v, |&i| *i % 2 == 0));
        assert_eq!(filtered, answer);
    }
}
