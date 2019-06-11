//! let's compare some manual implementations of collecting into a vector.
use rayon::scope;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use std::iter::repeat;

fn main() {
    let start = time::precise_time_ns();
    let v: Vec<u64> = (0..10_000_000u64)
        .into_par_iter()
        .filter(|&e| e % 3 == 0)
        .fold(
            || Vec::with_capacity(10_000_000),
            |mut v, e| {
                v.push(e);
                v
            },
        )
        .into_iter()
        .fold(None, |p: Option<Vec<_>>, v| {
            if let Some(mut previous_vec) = p {
                previous_vec.extend(v);
                Some(previous_vec)
            } else {
                Some(v)
            }
        })
        .unwrap();
    assert_eq!(v.first(), Some(&0));
    assert_eq!(v.last(), Some(&9_999_999));
    let end = time::precise_time_ns();
    println!("we took with sequential fusion: {}", end - start);

    let v: Vec<u64> = (0..10_000_000u64)
        .into_par_iter()
        .filter(|&e| e % 3 == 0)
        .fold(
            || Vec::with_capacity(10_000_000),
            |mut v, e| {
                v.push(e);
                v
            },
        )
        .into_iter()
        .fold(None, |p: Option<Vec<_>>, v| {
            if let Some(mut previous_vec) = p {
                let current_len = previous_vec.len();
                unsafe { previous_vec.set_len(current_len + v.len()) };
                (&mut previous_vec[current_len..])
                    .into_par_iter()
                    .zip(v.as_slice().into_par_iter())
                    .with_policy(Policy::Adaptive(1_000, 10_000))
                    .for_each(|(d, s)| *d = *s);
                Some(previous_vec)
            } else {
                Some(v)
            }
        })
        .unwrap();
    assert_eq!(v.first(), Some(&0));
    assert_eq!(v.last(), Some(&9_999_999));
    let end = time::precise_time_ns();
    println!("we took with parallel fusion: {}", end - start);

    let v: Vec<u64> = (0..10_000_000u64)
        .into_par_iter()
        .filter(|&e| e % 3 == 0)
        .fold(
            || Vec::with_capacity(10_000_000),
            |mut v, e| {
                v.push(e);
                v
            },
        )
        .into_iter()
        .fold(None, |p: Option<Vec<_>>, v| {
            if let Some(mut previous_vec) = p {
                let current_len = previous_vec.len();
                unsafe { previous_vec.set_len(current_len + v.len()) };
                (&mut previous_vec[current_len..])
                    .into_par_iter()
                    .zip(v.as_slice().into_par_iter())
                    .with_policy(Policy::Adaptive(1000, 10_000))
                    .cap(2)
                    .for_each(|(d, s)| *d = *s);
                Some(previous_vec)
            } else {
                Some(v)
            }
        })
        .unwrap();
    assert_eq!(v.first(), Some(&0));
    assert_eq!(v.last(), Some(&9_999_999));
    let end = time::precise_time_ns();
    println!("we took with adaptive fusion: {}", end - start);

    unimplemented!("there is quite a bug here");

    let v: Vec<u64> = scope(|s| {
        (0..10_000_000u64)
            .into_par_iter()
            .filter(|&e| e % 3 == 0)
            .by_blocks(repeat(2_000_000))
            .with_policy(Policy::Adaptive(1_000, 100_000))
            .with_help(|i| i.collect::<Vec<_>>())
            .fold(
                Vec::with_capacity(10_000_000),
                |mut v, e| {
                    v.push(e);
                    v
                },
                |mut v, v2| {
                    //TODO: use scope to fill in capped parallel
                    v.extend(v2);
                    v
                },
            )
    });
    assert_eq!(v.first(), Some(&0));
    assert_eq!(v.last(), Some(&9_999_999));
    let end = time::precise_time_ns();
    println!("we took with helper scheme : {}", end - start);
}
