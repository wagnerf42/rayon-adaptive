#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use std::iter::repeat_with;
use time::precise_time_ns;

fn fibo(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        fibo(n - 1) + fibo(n - 2)
    }
}

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("building pool failed");

    let t: Vec<(u64, u64)> = repeat_with(|| {
        let (t, d) = pool.logging_install(|| {
            let start = precise_time_ns();
            let x = fibo(10);
            assert!(x > 0);
            let end = precise_time_ns();
            (end - start) as u64
        });
        (t, d.duration)
    })
    .take(10_000)
    .collect();
    // for f in &t {
    //    println!("{:?}", f);
    // }
    eprintln!("it took {}", t.iter().map(|(t, _)| t).sum::<u64>());
    eprintln!("it took {}", t.iter().map(|(_, d)| d).sum::<u64>());
}
