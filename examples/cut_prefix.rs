//! example for `cut` : prefix algorithm

#[cfg(not(feature = "logs"))]
fn main() {
    eprintln!("please recompile with the \"logs\" feature")
}

#[cfg(feature = "logs")]
fn main() {
    use rayon_adaptive::prelude::*;
    use rayon_adaptive::Policy;
    use std::iter::repeat;
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .build()
        .expect("failed building pool");
    let mut v = repeat(1).take(1_000_000).collect::<Vec<u64>>();
    let (_, log) = pool.logging_install(|| {
        v.as_mut_slice()
            .cut()
            .map(|s| {
                s.iter_mut().fold(0, |acc, e| {
                    *e += acc;
                    *e
                });
                s
            })
            .log("partial sum")
            .reduced_iter()
            .fold(0, |acc, slice| {
                if acc != 0 {
                    slice.into_par_iter().log("update").for_each(|e| *e += acc);
                }
                slice.last().cloned().unwrap_or(acc)
            })
    });
    log.save_svg("parallel_prefix.svg")
        .expect("failed saving svg");
}
