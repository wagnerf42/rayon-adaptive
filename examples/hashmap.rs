extern crate rayon_adaptive;
use rayon_adaptive::par_keys;
use rayon_adaptive::prelude::*;
use std::collections::HashMap;

fn main() {
    let h: HashMap<u32, u32> = (0..1000).map(|i| (i, i + 1)).collect();
    let s: u32 = par_keys(&h).sum();
    assert_eq!(s, 500 * 999);
}
