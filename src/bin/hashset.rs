use rayon_adaptive::par_elements;
use rayon_adaptive::prelude::*;
use std::collections::HashSet;

fn main() {
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
