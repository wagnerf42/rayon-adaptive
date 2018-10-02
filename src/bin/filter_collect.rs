extern crate rand;
#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rand::random;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::{Divisible, EdibleSlice, EdibleSliceMut, Mergeable, Policy};
use std::collections::LinkedList;

struct FilterWork<'a> {
    input: EdibleSlice<'a, u32>,
    output: EdibleSliceMut<'a, u32>,
}

// we need to implement it manually to split output at best index.
impl<'a> Divisible for FilterWork<'a> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let (left_input, right_input) = self.input.split();
        let remaining_left_size = left_input.len();
        let (left_output, right_output) = self.output.split_at(remaining_left_size);
        (
            FilterWork {
                input: left_input,
                output: left_output,
            },
            FilterWork {
                input: right_input,
                output: right_output,
            },
        )
    }
}

fn filter_collect(slice: &[u32], policy: Policy) -> Vec<u32> {
    let size = slice.len();
    let mut uninitialized_output = Vec::with_capacity(size);
    unsafe {
        uninitialized_output.set_len(size);
    }
    let used = {
        let input = FilterWork {
            input: EdibleSlice::new(slice),
            output: EdibleSliceMut::new(uninitialized_output.as_mut_slice()),
        };
        let mut output_slices = input.work(
            |mut slices, limit| {
                for (i, o) in slices
                    .input
                    .iter()
                    .take(limit)
                    .filter(|&i| i % 2 == 0)
                    .zip(slices.output.iter_mut())
                {
                    *o = *i;
                }
                slices
            },
            |slices| {
                let mut l = LinkedList::new();
                l.push_back(slices.output);
                l
            },
            policy,
        );
        let first_output_slice = output_slices.pop_front().unwrap();
        let final_output =
            output_slices
                .into_iter()
                .fold(first_output_slice, |left_slice, right_slice| {
                    left_slice.fuse(right_slice) // TODO: this is done in src/slices.rs
                                                 // should we move it back here ?
                                                 // and also, should we do it in parallel ?
                });
        slice.len() - final_output.len()
    };
    unsafe {
        uninitialized_output.set_len(used);
    }
    uninitialized_output
}

fn main() {
    let v: Vec<u32> = (0..1_000_000).map(|_| random::<u32>() % 10).collect();
    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    #[cfg(feature = "logs")]
    {
        let (filtered, log) = pool.install(|| filter_collect(&v, Policy::Adaptive(2000)));
        assert_eq!(filtered, answer);
        log.save_svg("filter.svg").expect("failed saving svg");
    }
    #[cfg(not(feature = "logs"))]
    {
        let filtered = pool.install(|| filter_collect(&v, Policy::Adaptive(2000)));
        assert_eq!(filtered, answer);
    }
}
