extern crate rayon_adaptive;

use rayon_adaptive::prelude::*;

fn main() {
    let v: Vec<u32> = (0..20_000).collect();
    let s = v.as_slice();
    let sum = s.map_reduce(|s| s.iter().sum(), |r1: u32, r2| r1 + r2);
    assert_eq!(sum, 10_000 * 19_999);
}
