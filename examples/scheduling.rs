#[cfg(not(feature = "logs"))]
fn main() {
    eprintln!("please recompile with the \"logs\" feature")
}

#[cfg(feature = "logs")]
fn main() {
    use rayon_adaptive::prelude::*;
    use rayon_adaptive::Policy;
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .build()
        .expect("failed building pool");

    let (max, log) = pool.logging_install(|| {
        (0..100_000_000u64)
            .into_par_iter()
            .with_policy(Policy::Rayon(1))
            .max()
    });
    assert_eq!(max, Some(99_999_999));
    log.save_svg("max_rayon.svg")
        .expect("failed saving svg file");

    let (max, log) = pool.logging_install(|| {
        (0..100_000_000u64)
            .into_par_iter()
            .with_policy(Policy::Adaptive(1, 10_000))
            .max()
    });
    assert_eq!(max, Some(99_999_999));
    log.save_svg("max_adaptive.svg")
        .expect("failed saving svg file");

    let (max, log) = pool.logging_install(|| {
        (0..100_000_000u64)
            .into_par_iter()
            .with_policy(Policy::Join(100_000))
            .max()
    });
    assert_eq!(max, Some(99_999_999));
    log.save_svg("max_join.svg")
        .expect("failed saving svg file");
}
