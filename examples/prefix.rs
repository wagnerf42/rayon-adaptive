//! Example of use of the `work` method on `Divisible` input.
use rayon_adaptive::prelude::*;
use rayon_adaptive::IndexedPower;
use rayon_adaptive::Policy;
use std::iter::repeat;

struct PartialSlice<'a, T: 'a + Sync> {
    used: usize,
    slice: &'a mut [T],
}

impl<'a, T: 'a + Sync> Divisible<IndexedPower> for PartialSlice<'a, T> {
    fn base_length(&self) -> Option<usize> {
        Some(self.slice.len() - self.used)
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left_slice, right_slice) = self.slice.divide_at(self.used + index);
        self.slice = left_slice;
        (
            self,
            PartialSlice {
                used: 0,
                slice: right_slice,
            },
        )
    }
}

fn main() {
    let mut input: Vec<u64> = repeat(1).take(10).collect();
    let s = PartialSlice {
        used: 0,
        slice: input.as_mut_slice(),
    };
    //TODO: wip
    s.work(|mut s, size| {
        let previous_value = s.slice[..s.used].last().cloned().unwrap_or(0);
        s.slice[s.used..(s.used + size)]
            .iter_mut()
            .fold(previous_value, |v, e| {
                *e += v;
                *e
            });
        s.used += size;
        s
    })
    .with_policy(Policy::Join(5))
    .map(|s| s.slice)
    .into_iter()
    .fold(None, |possible_last, s| {
        println!("slice is : {:?}", s);
        if let Some(previous_value) = possible_last {
            s.iter_mut().for_each(|e| *e += previous_value)
        }
        s.last().clone()
    });
    assert_eq!((1..=10).collect::<Vec<u64>>(), input);
}
