extern crate rayon_adaptive;
use rayon_adaptive::prelude::*;

fn main() {
    let v: Vec<u32> = (0..10_000).collect();
    let s = v
        .into_adapt_iter()
        .fold(|| 0, |acc, x| acc + *x)
        .reduce(|a, b| a + b);
    assert_eq!(s, v.len() as u32 * (v.len() as u32 - 1) / 2);
    let s: usize = (0..10_000).into_adapt_iter().map(|x| x * 2).sum();
    assert_eq!(s, 10_000 * 9_999);
    let all_eq = (0..1000)
        .into_adapt_iter()
        .zip((0..1000).into_adapt_iter())
        .map(|(a, b)| a == b)
        .fold(|| true, |acc, t| if t { acc } else { false })
        .reduce(|a, b| a && b);
    assert!(all_eq);

    assert!((0..10_000).into_adapt_iter().any(|x| x == 2345));
}
