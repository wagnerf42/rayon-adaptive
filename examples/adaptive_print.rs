//! fully adaptive algorithm on iterators.
//! sequential thread is doing mapping+io (or io if data retrieved from helper)
//! helper threads are mapping+storage
use rayon_adaptive::prelude::*;
use std::iter::repeat;

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
    (1usize..100)
        .into_par_iter()
        .map(f)
        .by_blocks(repeat(50))
        .with_help(
            |i| i.for_each(|e| println!("{}", e)),
            |i| i.collect::<Vec<_>>(),
            |_, v| v.iter().for_each(|e| println!("{}", e)),
        )
}
