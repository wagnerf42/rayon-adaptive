extern crate itertools;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rayon_logs::ThreadPoolBuilder;

use itertools::kmerge;
use rayon_adaptive::{schedule, Block, Output, Policy};

/// We can now fuse contiguous slices together back into one.
fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.offset(s1.len() as isize) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

/// We'll need slices of several vectors at once.
struct SortingSlices<'a, T: 'a> {
    s: Vec<&'a mut [T]>, // we have 2 input slices and one output slice
    i: usize,            // index of slice containing the data
}

impl<'a, T: 'a> SortingSlices<'a, T> {
    /// Return the two mutable slices of given indices.
    fn mut_couple<'b>(&'b mut self, i1: usize, i2: usize) -> (&'b mut [T], &'b mut [T]) {
        let (s0, s1, s2) = {
            let (s0, leftover) = self.s.split_first_mut().unwrap();
            let (s1, s2) = leftover.split_first_mut().unwrap();
            (s0, s1, s2.get_mut(0).unwrap())
        };
        match (i1, i2) {
            (0, 1) => (s0, s1),
            (0, 2) => (s0, s2),
            (1, 0) => (s1, s0),
            (1, 2) => (s1, s2),
            (2, 0) => (s2, s0),
            (2, 1) => (s2, s1),
            _ => panic!("i1 == i2"),
        }
    }
}

impl<'a, T: 'a + Ord + Copy> Block for SortingSlices<'a, T> {
    type Output = SortingSlices<'a, T>;
    fn len(&self) -> usize {
        self.s[0].len()
    }
    fn split(self, i: usize) -> (Self, Self) {
        let v = self.s.into_iter().map(|s| s.split_at_mut(i)).fold(
            (Vec::new(), Vec::new()),
            |mut acc, (s1, s2)| {
                acc.0.push(s1);
                acc.1.push(s2);
                acc
            },
        );
        (
            SortingSlices { s: v.0, i: self.i },
            SortingSlices { s: v.1, i: self.i },
        )
    }
    fn compute(self) -> Self::Output {
        let mut slices = self;
        slices.s[slices.i].sort();
        slices
    }
}

impl<'a, T: 'a + Ord + Copy> Output for SortingSlices<'a, T> {
    fn fuse(self, other: Self) -> Self {
        let mut slices = self;
        let mut other = other;
        let destination = (0..3usize)
            .filter(|&i| i != slices.i && i != other.i)
            .next()
            .unwrap();
        {
            let i1 = slices.i;
            let (s1, d1) = slices.mut_couple(i1, destination);
            let i2 = other.i;
            let (s2, d2) = other.mut_couple(i2, destination);
            for (i, o) in kmerge(vec![s1, s2]).zip(d1.iter_mut().chain(d2.iter_mut())) {
                *o = *i
            }
        }
        SortingSlices {
            s: slices
                .s
                .into_iter()
                .zip(other.s.into_iter())
                .map(|(s1, s2)| fuse_slices(s1, s2))
                .collect(),
            i: destination,
        }
    }
}

fn generic_sort<T: Ord + Copy + Send>(v: &mut [T], policy: Policy) {
    let mut buffer1 = Vec::with_capacity(v.len());
    unsafe { buffer1.set_len(v.len()) }
    let mut buffer2 = Vec::with_capacity(v.len());
    unsafe { buffer2.set_len(v.len()) }
    let vectors = SortingSlices {
        s: vec![v, &mut buffer1, &mut buffer2],
        i: 0,
    };
    let mut result: SortingSlices<T> = schedule(vectors, policy);

    // we might get one extra copy at the end if data is not in the right buffer
    if result.i != 0 {
        let (final_slice, added_buffers) = result.s.split_first_mut().unwrap();
        final_slice.copy_from_slice(added_buffers[result.i - 1]);
    }
}

fn main() {
    let mut v: Vec<u32> = (0..100_000).collect();
    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("failed building pool");
    let log = pool.install(|| generic_sort(&mut v, Policy::Join(2000))).1;
    log.save_svg("join.svg").expect("saving svg file failed");
}
