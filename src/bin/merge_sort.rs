extern crate itertools;
extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::{ChaChaRng, Rng};
use rayon_logs::ThreadPoolBuilder;

use itertools::kmerge;
use rayon_adaptive::{schedule, Block, Output, Policy};

trait Boolean {
    fn value() -> bool;
}
struct True;
struct False;
impl Boolean for True {
    fn value() -> bool {
        true
    }
}
impl Boolean for False {
    fn value() -> bool {
        false
    }
}

fn partial_manual_merge<
    CheckLeft: Boolean,
    CheckRight: Boolean,
    CheckLimit: Boolean,
    T: Ord + Copy,
>(
    input1: &[T],
    input2: &[T],
    output: &mut [T],
    limit: usize,
) -> Option<(usize, usize, usize)> {
    let mut i1 = 0;
    let mut i2 = 0;
    let mut i_out = 0;
    let (check_left, check_right, check_limit) =
        (CheckLeft::value(), CheckRight::value(), CheckLimit::value());
    if check_limit {
        debug_assert_eq!(true, input1.len() + input2.len() >= limit);
        debug_assert_eq!(true, limit <= output.len());
    } else {
        debug_assert_eq!(input1.len() + input2.len(), output.len());
    }
    if check_left && input1.is_empty() {
        output.copy_from_slice(input2);
        return None;
    } else if check_right && input2.is_empty() {
        output.copy_from_slice(input1);
        return None;
    } else {
        unsafe {
            let mut value1 = input1.get_unchecked(i1);
            let mut value2 = input2.get_unchecked(i2);
            for o in output.iter_mut() {
                if check_limit && i_out >= limit {
                    break;
                }
                if value1.lt(value2) {
                    *o = *value1;
                    i1 += 1;
                    i_out += 1;
                    if check_left && i1 >= input1.len() {
                        break;
                    }
                    value1 = input1.get_unchecked(i1);
                } else {
                    *o = *value2;
                    i2 += 1;
                    i_out += 1;
                    if check_right && i2 >= input2.len() {
                        break;
                    }
                    value2 = input2.get_unchecked(i2);
                }
            }
        }
        if check_right && i2 == input2.len() {
            output[(i1 + i2)..].copy_from_slice(&input1[i1..]);
            return None;
        } else if check_left && i1 == input1.len() {
            output[(i1 + i2)..].copy_from_slice(&input2[i2..]);
            return None;
        }
    }
    Some((i1, i2, i_out))
}

/// We can now fuse contiguous slices together back into one.
fn fuse_slices<'a, 'b, 'c: 'a + 'b, T: 'c>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.offset(s1.len() as isize) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

/// Adaptive merge input
struct MergeBlock<'a, T: 'a> {
    left: &'a [T],
    right: &'a [T],
    output: &'a mut [T],
}

struct MergeOutput();

impl Output for MergeOutput {
    fn fuse(self, other: Self) -> Self {
        MergeOutput()
    }
}

impl<'a, T: 'a + Ord + Copy> Block for MergeBlock<'a, T> {
    type Output = MergeOutput;
    fn len(&self) -> usize {
        self.output.len()
    }
    fn split(self, i: usize) -> (Self, Self) {
        unimplemented!()
    }
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output) {
        partial_manual_merge::<True, True, False, _>(self.left, self.right, self.output, limit);
        unimplemented!("we need to figure out what's left")
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
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output) {
        unimplemented!()
        //   let mut slices = self;
        //   slices.s[slices.i][].sort();
        //   slices
    }
}

impl<'a, T: 'a + Ord + Copy> Output for SortingSlices<'a, T> {
    fn len(&self) -> usize {
        self.s[0].len()
    }
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
    let answer = v.clone();
    let mut ra = ChaChaRng::new_unseeded();
    ra.shuffle(&mut v);

    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("failed building pool");
    let log = pool.install(|| generic_sort(&mut v, Policy::Adaptive(2000, 2.0)))
        .1;
    assert_eq!(v, answer);
    log.save_svg("adapt.svg").expect("failed saving svg");
    //    pool.compare(
    //        "join",
    //        "join_context",
    //        || {
    //            let mut w = v.clone();
    //            generic_sort(&mut w, Policy::Join(2000))
    //        },
    //        || {
    //            let mut w = v.clone();
    //            generic_sort(&mut w, Policy::DepJoin(2000))
    //        },
    //        "joins_battle.html",
    //    ).expect("saving logs failed");
}
