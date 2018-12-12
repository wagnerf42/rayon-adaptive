use rayon_adaptive::par_iter;
use rayon_adaptive::prelude::*;
use std::collections::HashMap;

fn main() {
    let mut hash_map = HashMap::new();
    hash_map.insert(0, (0, 0));
    hash_map.insert(1, (0, 1));
    hash_map.insert(2, (1, 0));
    hash_map.insert(3, (0, 2));
    hash_map.insert(4, (2, 0));
    let my_vec: Vec<_> = par_iter(&hash_map)
        .filter(|(index, (_, _))| **index > 0)
        .collect();
    println!("{:?}", my_vec);
}
