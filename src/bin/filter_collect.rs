extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{schedule, Block, Output, Policy};
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

impl<'a> Block for FilterInput<'a> {
    type Output = FilterOutput<'a>;
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.input.len();
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
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output) {
        unimplemented!()
    }
}

impl<'a> Output for FilterOutput<'a> {
    fn fuse(self, other: Self) -> Self {
        self.slice[self.used..].copy_from_slice(&other.slice[other.used..]);
        FilterOutput {
            slice: fuse_slices(self.slice, other.slice),
            used: self.used + other.used,
        }
    }
}

fn filter_collect(slice: &[u32], policy: Policy) -> Vec<u32> {
    unimplemented!()
}

fn main() {
    let v: Vec<u32> = (0..100_000).map(|_| random::<u32>() % 2).collect();
    let answer: Vec<u32> = v.iter().filter(|&i| i % 2 == 0).cloned().collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    let (filtered, log) = pool.install(|| filter_collect(&v, Policy::Adaptive(2000)));
    assert_eq!(filtered, answer);
    log.save_svg("filter.svg").expect("failed saving svg");
}
