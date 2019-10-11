//! Parallel deterministic random numbers
use rayon_adaptive::chacha_iter;
use rayon_adaptive::prelude::*;

fn main() {
    let seed = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let v: Vec<u64> = chacha_iter(seed).map(|e: u64| e % 10).take(10).collect();
    eprintln!("v is {:?}", v);
}
