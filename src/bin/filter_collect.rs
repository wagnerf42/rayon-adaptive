use rayon::prelude::*;

extern crate rand;
extern crate rayon_adaptive;
#[cfg(not(feature = "logs"))]
use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
#[cfg(feature = "logs")]
use rayon_logs::ThreadPoolBuilder;

#[cfg(feature = "logs")]
use rayon_logs::scope;

#[cfg(feature = "logs")]
use rayon_logs::subgraph;

#[cfg(not(feature = "logs"))]
use rayon::scope;

use rand::random;
use std::iter::repeat;

fn even(e: &&u32) -> bool {
    **e % 2 == 0
}

fn fully_adaptive_test(v: &[u32], answer: &[u32]) {
    let mut a = Vec::with_capacity(2_000_000);
    unsafe { a.set_len(2_000_000) };
    scope(|sc| {
        v.into_adapt_iter()
            .filter(even)
            .cloned()
            .by_blocks(repeat(500_000))
            .fold(
                || Vec::with_capacity(500_000),
                |mut v, e| {
                    v.push(e);
                    v
                },
            )
            .helping_fold(
                (a.as_mut_slice(), 0),
                |(s, i), e| {
                    s[i] = e;
                    (s, i + 1)
                },
                |(s, i), v2| {
                    let (his, mine) = s[i..].split_at_mut(v2.len());
                    sc.spawn(move |_| {
                        subgraph("retrieve", v2.len(), || {
                            for (i, o) in v2.iter().zip(his.iter_mut()) {
                                *o = *i;
                            }
                        })
                    });
                    (mine, 0)
                },
            );
    });
    assert_eq!(a, answer);
}

fn main() {
    let v: Vec<u32> = (0..4_000_000).collect();
    let answer: Vec<u32> = v.iter().filter(even).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");

    #[cfg(feature = "logs")]
    {
        pool.compare()
            .attach_algorithm_nodisplay("rayon", || {
                assert_eq!(
                    answer,
                    v.par_iter().filter(even).cloned().collect::<Vec<_>>()
                )
            })
            .attach_algorithm("adaptive", || {
                assert_eq!(
                    answer,
                    v.into_adapt_iter()
                        .filter(even)
                        .cloned()
                        .collect::<Vec<_>>()
                )
            })
            .attach_algorithm("fully_adaptive", || fully_adaptive_test(&v, &answer))
            .generate_logs("filter_collect.html")
            .expect("generating logs failed");
    }
    #[cfg(not(feature = "logs"))]
    {
        let filtered: Vec<_> = pool.install(|| v.into_adapt_iter().filter(even).cloned().collect());
        assert_eq!(filtered, answer);
    }
}
