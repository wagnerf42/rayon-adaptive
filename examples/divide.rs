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
        let i = (0..1_000_000u64)
            .into_par_iter()
            .map(|e| e * 2)
            .with_policy(Policy::Join(100_000));
        let (i_start, i_end) = i.divide_at(300_000);
        assert_eq!(i_start.to_sequential().sum::<u64>(), 300_000 * 299_999);
        i_end.log("max").max()
    });
    assert_eq!(max, Some(2 * 999_999));
    log.save_svg("partial_max.svg")
        .expect("failed saving svg file");
}
