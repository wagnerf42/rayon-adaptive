//! this is a false merge sort algorithm just to explain `cut`.
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;

fn main() {
    let mut input = (1..100).rev().collect::<Vec<u32>>();
    input
        .as_mut_slice()
        .cut()
        .map(|s| {
            s.sort();
            s
        })
        .with_policy(Policy::Join(20))
        .for_each(|s| println!("our part is {:?}", s));
}
