use rayon_adaptive::prelude::*;
use std::ops::Index;

fn main() {
    let even: Vec<u32> = (0..10u32).map(|e| 2 * e).collect();
    let odds: Vec<u32> = (0..10u32).map(|e| 2 * e + 1).collect();
    let v: Vec<u32> = even.par_iter().merge(odds.par_iter()).cloned().collect();
    println!("we have: {:?}", v);
}
