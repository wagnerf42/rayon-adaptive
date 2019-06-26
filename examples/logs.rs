use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

#[cfg(not(feature = "logs"))]
fn main() {
    eprintln!("please recompile with the \"logs\" feature")
}

#[cfg(feature = "logs")]
fn main() {
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .build()
        .expect("failed building pool");
    let (max, log) = pool.logging_install(|| {
        (0..1_000_000u64)
            .into_par_iter()
            .fine_log("max")
            .with_policy(Policy::Adaptive(1000, 10_000))
            .max()
    });
    assert_eq!(max, Some(999_999));
    log.save_svg("fine_logs.svg")
        .expect("failed saving svg file");
}
