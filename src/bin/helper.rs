#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use std::iter::repeat;

use rayon_adaptive::prelude::*;

fn f(e: usize) -> usize {
    let mut c = 0;
    // let's try wasting some cpu
    for y in 0..e {
        for x in 0..e {
            c += x;
        }
        if c % 2 == 0 {
            c += y;
        }
    }
    c
}

fn main() {
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");

    pool.install(|| {
        (0..10_000)
            .into_adapt_iter()
            .map(|e| f(e))
            .by_blocks(repeat(5000))
            .fold(Vec::new, |mut v, e| {
                v.push(e);
                v
            })
            .helping_for_each(
                |e| println!("{}", e),
                |v| {
                    for e in v {
                        println!("{}", e);
                    }
                },
            )
    })
}
