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
                let current_len = previous_vec.len();
                unsafe { previous_vec.set_len(current_len + v.len()) };
                (&mut previous_vec[current_len..])
                    .into_par_iter()
                    .zip(v.as_slice().into_par_iter())
                    .with_policy(Policy::Adaptive(1000, 100_000))
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

    let start = time::precise_time_ns();
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
                    let current_length = v.len();
                    let reserved_slice = unsafe {
                        v.set_len(current_length + v2.len());
                        std::slice::from_raw_parts_mut(
                            v.as_mut_slice().as_mut_ptr().add(current_length),
                            v2.len(),
                        )
                    };
                    s.spawn(move |_s| {
                        reserved_slice
                            .into_par_iter()
                            .zip(v2.as_slice().into_par_iter())
                            .with_policy(Policy::Adaptive(100, 100_000))
                            //                            .cap(2) // the cap will not work unless we change rayon or stop being
                            //                            recursive in adaptive algorithms with a scope
                            .for_each(|(d, s)| *d = *s)
                    });
                    v
                },
            )
    });
    assert_eq!(v.first(), Some(&0));
    assert_eq!(v.last(), Some(&9_999_999));
    let end = time::precise_time_ns();
    println!("we took with helper scheme : {}", end - start);
}
