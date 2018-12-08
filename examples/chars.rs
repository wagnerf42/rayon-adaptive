extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;
fn main() {
    let s = "hello world";
    assert!(s.adapt_chars().all(|c| c != 'a'));
}
