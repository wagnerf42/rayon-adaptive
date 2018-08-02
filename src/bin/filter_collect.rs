extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{schedule, Block, Divisible, Output, Policy};
use rayon_logs::ThreadPoolBuilder;

/// We can now fuse contiguous slices together back into one.
fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.offset(s1.len() as isize) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

struct FilterInput<'a> {
    input: &'a [u32],
    output: &'a mut [u32],
}

struct FilterOutput<'a> {
    slice: &'a mut [u32],
    used: usize, // size really used from start
}

impl<'a> Divisible for FilterInput<'a> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.input.len() / 2;
        let (input_left, input_right) = self.input.split_at(mid);
        let (output_left, output_right) = self.output.split_at_mut(mid);
        (
            FilterInput {
                input: input_left,
                output: output_left,
            },
            FilterInput {
                input: input_right,
                output: output_right,
            },
        )
    }
}

impl<'a> Block for FilterInput<'a> {
    type Output = FilterOutput<'a>;
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output) {
        let mut collected = 0;
        for (i, o) in self.input
            .iter()
            .take(limit)
            .filter(|&i| i % 2 == 0)
            .zip(self.output.iter_mut())
        {
            *o = *i;
            collected += 1;
        }
        let remaining_input = &self.input[limit..];
        if remaining_input.is_empty() {
            (
                None,
                FilterOutput {
                    slice: self.output, // give back all slice to avoid holes
                    used: collected,
                },
            )
        } else {
            let (done_output, remaining_output) = self.output.split_at_mut(collected);
            (
                Some(FilterInput {
                    input: remaining_input,
                    output: remaining_output,
                }),
                FilterOutput {
                    slice: done_output,
                    used: collected,
                },
            )
        }
    }
}

impl<'a> Output for FilterOutput<'a> {
    fn fuse(self, other: Self) -> Self {
        if self.slice.len() >= self.used + other.used && self.slice.len() != self.used {
            // enough space to move data back and moving back required
            self.slice[self.used..(self.used + other.used)]
                .copy_from_slice(&other.slice[..other.used])
        }
        if self.slice.len() >= self.used + other.used || self.slice.len() == self.used {
            FilterOutput {
                slice: fuse_slices(self.slice, other.slice),
                used: self.used + other.used,
            }
        } else {
            // hard case, move things by hand
            let mut j = self.slice.len();
            let slice = fuse_slices(self.slice, other.slice);
            for i in (self.used)..(self.used + other.used) {
                slice[i] = slice[j];
                j += 1;
            }
            FilterOutput {
                slice,
                used: self.used + other.used,
            }
        }
    }
}

fn filter_collect(slice: &[u32], policy: Policy) -> Vec<u32> {
    let size = slice.len();
    let mut uninitialized_output = Vec::with_capacity(size);
    unsafe {
        uninitialized_output.set_len(size);
    }
    let used = {
        let input = FilterInput {
            input: slice,
            output: uninitialized_output.as_mut_slice(),
        };
        let output = schedule(input, policy);
        output.used
    };
    unsafe {
        uninitialized_output.set_len(used);
    }
    uninitialized_output
}

fn main() {
    let v: Vec<u32> = (0..100_000).map(|_| random::<u32>() % 10).collect();
    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    let (filtered, log) = pool.install(|| filter_collect(&v, Policy::Adaptive(2000)));
    assert_eq!(filtered, answer);
    log.save_svg("filter.svg").expect("failed saving svg");
}
