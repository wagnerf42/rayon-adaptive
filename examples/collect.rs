extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;

fn main() {
    let v: Vec<usize> = (0..10_000_000)
        .into_adapt_iter()
        .filter(|&x| x % 2 == 0)
        .collect();
    let v_seq: Vec<usize> = (0..10_000_000).filter(|&x| x % 2 == 0).collect();
    assert_eq!(v, v_seq);
}
