extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{fuse_slices, AdaptiveWork, Mergeable, Policy};
use rayon_logs::ThreadPoolBuilder;

struct FilterWork<'a> {
    input_slice: &'a [u32],
    output_slice: &'a mut [u32],
    used_input: usize,
    used_output: usize,
}

struct FilterOutput<'a> {
    slice: &'a mut [u32],
    used: usize,
}

//TODO: this version generates a lot of movements
//it would surely be better to use linked lists
impl<'a> Mergeable for FilterOutput<'a> {
    fn fuse(self, other: Self) -> Self {
        if self.slice.len() == self.used {
            FilterOutput {
                slice: fuse_slices(self.slice, other.slice),
                used: self.used + other.used,
            }
        } else {
            // move things back
            // TODO: we could do better by moving with copy in some cases
            let second_start = self.slice.len();
            let final_slice = fuse_slices(self.slice, other.slice);
            for i in 0..other.used {
                final_slice[self.used + i] = final_slice[second_start + i];
            }
            FilterOutput {
                slice: final_slice,
                used: self.used + other.used,
            }
        }
    }
}

impl<'a> AdaptiveWork for FilterWork<'a> {
    type Output = FilterOutput<'a>;
    fn work(&mut self, limit: usize) {
        let mut count = 0;
        for (i, o) in self.input_slice[self.used_input..]
            .iter()
            .filter(|&i| i % 2 == 0)
            .zip(self.output_slice[self.used_output..].iter_mut())
        {
            *o = *i;
            count += 1;
        }
        self.used_input += limit;
        self.used_output += count;
    }
    fn output(self) -> Self::Output {
        FilterOutput {
            slice: self.output_slice,
            used: self.used_output,
        }
    }
    fn remaining_length(&self) -> usize {
        self.input_slice.len() - self.used_input
    }
    fn split(self) -> (Self, Self) {
        let input_mid = self.used_input + (self.input_slice.len() - self.used_input) / 2;
        let (my_half_input, his_half_input) = self.input_slice.split_at(input_mid);
        let output_mid = self.used_output + my_half_input.len();
        let (my_half_output, his_half_output) = self.output_slice.split_at_mut(output_mid);
        (
            FilterWork {
                input_slice: my_half_input,
                output_slice: my_half_output,
                used_input: self.used_input,
                used_output: self.used_output,
            },
            FilterWork {
                input_slice: his_half_input,
                output_slice: his_half_output,
                used_input: 0,
                used_output: 0,
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
            input_slice: slice,
            output_slice: uninitialized_output.as_mut_slice(),
            used_input: 0,
            used_output: 0,
        };
        let output = input.schedule(policy);
        output.used
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
    let (filtered, log) = pool.install(|| filter_collect(&v, Policy::JoinContext(2000)));
    assert_eq!(filtered, answer);
    log.save_svg("filter.svg").expect("failed saving svg");
}
