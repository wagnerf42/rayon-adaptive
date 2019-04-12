use rayon::prelude::*;
use rayon_adaptive::prelude::*;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use std::iter::successors;

use rayon::ThreadPoolBuilder;

fn main() {
    let v: Vec<u32> = (0..10_000_000).collect();
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("failed building thread pool");
    let target = 4_800_000;

    let fold_reduce = || {
        assert_eq!(
            v.as_slice()
                .into_adapt_iter()
                .fold(
                    || None,
                    |opt, e| opt.or_else(|| if *e == target { Some(*e) } else { None })
                )
                .reduce(|o1, o2| o1.or(o2))
                .unwrap(),
            target
        )
    };

    let slice_reduce = || {
        assert_eq!(
            v.as_slice()
                .cutting_fold(
                    || None,
                    |opt, s| opt.or_else(|| s.iter().find(|&e| *e == target).cloned())
                )
                .reduce(|o1, o2| o1.or(o2))
                .unwrap(),
            target
        )
    };

    let blocked_reduce = || {
        assert_eq!(
            v.as_slice()
                .by_blocks(successors(Some(100), |i| Some(i * 2)))
                .cutting_fold(
                    || None,
                    |opt, s| opt.or_else(|| s.iter().find(|&e| *e == target).cloned())
                )
                .into_iter()
                .filter_map(|o| o)
                .next()
                .unwrap(),
            target
        )
    };

    let adapt = || {
        assert_eq!(
            v.as_slice()
                .by_blocks(successors(Some(100), |i| Some(i * 2)))
                .cutting_fold(
                    || None,
                    |opt, s| opt.or_else(|| s.iter().find(|&e| *e == target).cloned())
                )
                .helping_cutting_fold(
                    None,
                    |opt, s| opt.or_else(|| s.iter().find(|&e| *e == target).cloned()),
                    |o1, o2| o1.or(o2)
                )
                .unwrap(),
            target
        )
    };

    #[cfg(feature = "logs")]
    {
        pool.compare()
            // .attach_algorithm_nodisplay("rayon", || {
            //     assert_eq!(
            //         v.par_iter().find_first(|&e| *e == target).cloned().unwrap(),
            //         target
            //     )
            // })
            .attach_algorithm("fold_reduce", fold_reduce)
            .attach_algorithm("slice_reduce", slice_reduce)
            .attach_algorithm("blocked_reduce", blocked_reduce)
            .attach_algorithm("adapt", adapt)
            .generate_logs("find_first.html")
            .expect("failed saving find first comparisons");
    }
    #[cfg(not(feature = "logs"))]
    {
        pool.install(fold_reduce);
        pool.install(slice_reduce);
        pool.install(blocked_reduce);
        pool.install(adapt);
    }
}
