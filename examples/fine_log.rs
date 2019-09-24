extern crate rayon_adaptive;
#[cfg(feature = "logs")]
extern crate rayon_logs;

#[cfg(feature = "logs")]
fn main() {
    use rayon_adaptive::prelude::*;
    use rayon_logs::ThreadPoolBuilder;
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("failed creating pool");
    let (_, log) = pool.logging_install(|| {
        let s: u32 = (0u32..10_000)
            .into_par_iter()
            .chain(0u32..1_000)
            .with_join_policy(10_000)
            //            .even_levels()
            .fine_log("sum")
            .sum();
        assert_eq!(s, 9_999 * 5_000 + 999 * 500);
    });
    log.save_svg("fine_log.svg")
        .expect("failed saving svg file");
}

#[cfg(not(feature = "logs"))]
fn main() {
    eprintln!("please run me with the logs enabled!");
}
