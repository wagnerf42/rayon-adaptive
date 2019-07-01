//! compare different versions of filter collect.

#[cfg(not(feature = "logs"))]
fn main() {
    eprintln!("please recompile with the \"logs\" feature")
}

#[cfg(feature = "logs")]
fn main() {
    use rayon_adaptive::prelude::*;
    use rayon_adaptive::Policy;
    use std::iter::repeat;
    let pool = rayon_logs::ThreadPoolBuilder::new()
        .num_threads(3)
        .build()
        .expect("failed building pool");

    pool.compare()
        .attach_algorithm("map_reduce", || {
            let r: Vec<u64> = (0..10_000_000u64)
                .into_par_iter()
                .filter(|&e| e % 2 == 0)
                .map(|e| vec![e])
                .reduce(Vec::new, |mut v1, v2| {
                    v1.extend(v2);
                    v1
                });
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&9_999_998));
        })
        .attach_algorithm("fold_reduce", || {
            let r: Vec<u64> = (0..10_000_000u64)
                .into_par_iter()
                .filter(|&e| e % 2 == 0)
                .fold(Vec::new, |mut v, e| {
                    v.push(e);
                    v
                })
                .reduce(Vec::new, |mut v1, v2| {
                    v1.extend(v2);
                    v1
                });
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&9_999_998));
        })
        .attach_algorithm("adaptive", || {
            let r: Vec<u64> = (0..10_000_000u64)
                .into_par_iter()
                .filter(|&e| e % 2 == 0)
                .with_policy(Policy::Adaptive(1_000, 50_000))
                .with_help(|i| i.collect::<Vec<_>>())
                .fold(
                    Vec::with_capacity(5_000_000),
                    |mut v, e| {
                        v.push(e);
                        v
                    },
                    |mut v1, v2| {
                        v1.extend(v2);
                        v1
                    },
                );
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&9_999_998));
        })
        .attach_algorithm("list", || {
            let mut i = (0..10_000_000u64)
                .into_par_iter()
                .filter(|&e| e % 2 == 0)
                .fold(Vec::new, |mut v, e| {
                    v.push(e);
                    v
                })
                .reduced_iter();
            let mut r = i.next().unwrap_or_else(Vec::new);
            for block in i {
                r.extend(block);
            }
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&9_999_998));
        })
        .attach_algorithm("adaptive_plus", || {
            let r: Vec<u64> = rayon_logs::scope(|s| {
                (0..10_000_000u64)
                    .into_par_iter()
                    .filter(|&e| e % 2 == 0)
                    .with_policy(Policy::Adaptive(1_000, 50_000))
                    .by_blocks(repeat(2_000_000))
                    .with_help(|i| i.collect::<Vec<_>>())
                    .fold(
                        Vec::with_capacity(5_000_000),
                        |mut v, e| {
                            v.push(e);
                            v
                        },
                        |mut v1, v2| {
                            let current_len = v1.len();
                            unsafe { v1.set_len(current_len + v2.len()) };
                            let borrowed_slice = unsafe {
                                std::slice::from_raw_parts_mut(
                                    v1.as_mut_ptr().offset(current_len as isize),
                                    v2.len(),
                                )
                            };
                            s.spawn(move |_| {
                                borrowed_slice
                                    .into_par_iter()
                                    .zip(v2.as_slice().into_par_iter())
                                    .with_policy(Policy::Adaptive(1_000, 50_000))
                                    .for_each(|(d, s)| *d = *s)
                            });
                            v1
                        },
                    )
            });
            assert_eq!(r.first(), Some(&0));
            assert_eq!(r.last(), Some(&9_999_998));
        })
        .generate_logs("filter_collect.html")
        .expect("writing logs failed");
}
