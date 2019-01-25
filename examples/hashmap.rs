extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;
use rayon_adaptive::{par_elements, par_keys};
use std::collections::{HashMap, HashSet};

fn main() {
    let h: HashMap<u32, u32> = (0..1000).map(|i| (i, i + 1)).collect();
    let s: u32 = par_keys(&h).sum();
    assert_eq!(s, 500 * 999);
    let mut hash_set = HashSet::new();
    hash_set.insert(0);
    hash_set.insert(0);
    hash_set.insert(1);
    hash_set.insert(1);
    hash_set.insert(2);
    hash_set.insert(3);
    let my_vec: Vec<_> = par_elements(&hash_set)
        .filter(|index| **index > 0)
        .collect();
    println!("{:?}", my_vec);
}
