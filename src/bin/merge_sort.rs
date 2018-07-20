extern crate itertools;
extern crate rayon_adaptive;
use itertools::kmerge;
use rayon_adaptive::{schedule, Policy};

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
    /// pre-condition: i1 < i2.
    fn mut_couple<'b>(&'b mut self, i1: usize, i2: usize) -> (&'b mut [T], &'b mut [T]) {
        let (s0, s1, s2) = {
            let (s0, leftover) = self.s.split_first_mut().unwrap();
            let (s1, s2) = leftover.split_first_mut().unwrap();
            (s0, s1, s2.get_mut(0).unwrap())
        };
        match (i1, i2) {
            (0, 1) => (s0, s1),
            (0, 2) => (s0, s2),
            (1, 2) => (s1, s2),
            _ => panic!("precondition is not ok"),
        }
    }
}

fn split_at_mut<'a, T: 'a>(
    slices: SortingSlices<'a, T>,
    i: usize,
) -> (SortingSlices<'a, T>, SortingSlices<'a, T>) {
    let v = slices.s.into_iter().map(|s| s.split_at_mut(i)).fold(
        (Vec::new(), Vec::new()),
        |mut acc, (s1, s2)| {
            acc.0.push(s1);
            acc.1.push(s2);
            acc
        },
    );
    (
        SortingSlices {
            s: v.0,
            i: slices.i,
        },
        SortingSlices {
            s: v.1,
            i: slices.i,
        },
    )
}

fn fuse<'a, T: 'a + Ord + Copy>(
    slices: SortingSlices<'a, T>,
    other: SortingSlices<'a, T>,
) -> SortingSlices<'a, T> {
    let mut slices = slices;
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

fn final_sort<'a, T: Ord>(mut slices: SortingSlices<'a, T>) -> SortingSlices<'a, T> {
    slices.s[slices.i].sort();
    slices
}

fn generic_sort<T: Ord + Copy>(v: &mut [T], policy: Policy) {
    let mut buffer1 = Vec::with_capacity(v.len());
    unsafe { buffer1.set_len(v.len()) }
    let mut buffer2 = Vec::with_capacity(v.len());
    unsafe { buffer2.set_len(v.len()) }
    let vectors = SortingSlices {
        s: vec![v, &mut buffer1, &mut buffer2],
        i: 0,
    };
    let mut result: SortingSlices<T> = schedule(vectors, split_at_mut, fuse, final_sort, policy);

    // we might get one extra copy at the end if data is not in the right buffer
    if result.i != 0 {
        let (final_slice, added_buffers) = result.s.split_first_mut().unwrap();
        final_slice.copy_from_slice(added_buffers[result.i - 1]);
    }
}

fn main() {
    let mut v: Vec<u32> = (0..20).collect();
    generic_sort(&mut v, Policy::Join(2000));
    println!("v: {:?}", v);
}
