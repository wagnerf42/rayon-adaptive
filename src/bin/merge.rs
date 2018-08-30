//! let's rewrite parallel merge before parallel merge sort
extern crate itertools;
extern crate rayon_adaptive;
extern crate rayon_logs;

use rayon_adaptive::{Divisible, EdibleSlice, EdibleSliceMut, Policy};
use rayon_logs::ThreadPoolBuilder;

struct FusionSlice<'a, T: 'a> {
    left: EdibleSlice<'a, T>,
    right: EdibleSlice<'a, T>,
    output: EdibleSliceMut<'a, T>,
}

impl<'a, T: 'a + Send + Sync> Divisible for FusionSlice<'a, T> {
    fn len(&self) -> usize {
        self.output.len()
    }
    fn split(self) -> (Self, Self) {
        unimplemented!()
    }
}

fn fuse<T: Ord + Send + Sync + Copy>(left: &[T], right: &[T], output: &mut [T]) {
    let slices = FusionSlice {
        left: EdibleSlice::new(left),
        right: EdibleSlice::new(right),
        output: EdibleSliceMut::new(output),
    };

    slices.work(
        |slices, limit| {
            let mut left_i = slices.left.iter();
            let mut right_i = slices.right.iter();
            for o in slices.output.iter_mut().take(limit) {
                let go_left = match (left_i.peek(), right_i.peek()) {
                    (Some(l), Some(r)) => l <= r,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    (None, None) => panic!("not enough input when merging"),
                };
                *o = if go_left {
                    *left_i.next().unwrap()
                } else {
                    *right_i.next().unwrap()
                };
            }
        },
        |_| (),
        Policy::Adaptive(10_000),
    );
}

fn main() {
    let v: Vec<u32> = (0..1_000_000).collect();
    let even: Vec<_> = v.iter().filter(|&i| *i % 2 == 0).cloned().collect();
    let odd: Vec<_> = v.iter().filter(|&i| *i % 2 == 1).cloned().collect();
    let mut w = vec![0; 1_000_000];

    let pool = ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .expect("pool creation failed");
    let log = pool.install(|| {
        fuse(&even, &odd, &mut w);
    }).1;
    log.save_svg("merge.svg").expect("saving svg failed");

    assert_eq!(v, w);
}
