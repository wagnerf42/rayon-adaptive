use crate::base::slice;
use crate::iter::EvenLevels;
use crate::prelude::*;
use itertools::merge;
use std::iter::{once, Once};
use std::slice::from_raw_parts_mut;

/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
fn fuse_slices<'a: 'c, 'b: 'c, 'c, T: 'a + 'b>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

struct SortingTuple<'a, T: 'a> {
    //ASK I want to remove this option, but cut_at_index complains about lifetimes.
    //I think it wants me to specify a lifetime for the borrow in the input args.
    //But, I can't do this, because I don't want to have this trait parameterized over a lifetime.
    //
    //It is really surprising how this even works with an Option. It should have the same error
    //about the lifetimes as before.
    //Are we fooling the compiler?
    inp_slice: Option<&'a mut [T]>,
    outp_slice: Option<&'a mut [T]>,
}

impl<'a, T: Send> DivisibleParallelIterator for SortingTuple<'a, T> {
    fn base_length(&self) -> usize {
        debug_assert!(
            self.inp_slice.as_ref().unwrap().len() == self.outp_slice.as_ref().unwrap().len()
        );
        self.inp_slice.as_ref().unwrap().len()
    }
    fn cut_at_index(&mut self, index: usize) -> Self {
        let (left_inp, right_inp) = self.inp_slice.take().unwrap().split_at_mut(index);
        let (left_outp, right_outp) = self.outp_slice.take().unwrap().split_at_mut(index);
        self.inp_slice = Some(right_inp);
        self.outp_slice = Some(right_outp);
        SortingTuple {
            inp_slice: Some(left_inp),
            outp_slice: Some(left_outp),
        }
    }
}

impl<'a, T: Send> IntoIterator for SortingTuple<'a, T> {
    type Item = SortingTuple<'a, T>;
    type IntoIter = Once<SortingTuple<'a, T>>;
    fn into_iter(self) -> Self::IntoIter {
        once(self)
    }
}

pub fn merge_sort_adaptive<'a, T: 'a + Send + Ord + Copy>(input: &'a mut [T]) {
    let mut copy_vector: Vec<T> = Vec::with_capacity(input.len());
    unsafe {
        copy_vector.set_len(input.len());
    }
    let to_sort = SortingTuple {
        inp_slice: Some(input),
        outp_slice: Some(&mut copy_vector),
    };

    to_sort
        .into_par_iter()
        .map(|mut s| {
            s.inp_slice.as_mut().map(|inner_slice| inner_slice.sort());
            s
        })
        .with_join_policy(2000)
        .with_rayon_policy()
        .even_levels()
        .reduce(
            || SortingTuple {
                inp_slice: None,
                outp_slice: None,
            },
            |mut left_sorted, mut right_sorted| {
                if left_sorted.inp_slice.is_none() {
                    right_sorted
                } else if right_sorted.inp_slice.is_none() {
                    left_sorted
                } else {
                    let left_input = left_sorted.inp_slice.take().unwrap();
                    let right_input = right_sorted.inp_slice.take().unwrap();

                    let left_output = left_sorted.outp_slice.take().unwrap();
                    let right_output = right_sorted.outp_slice.take().unwrap();
                    let mut new_output = fuse_slices(left_output, right_output);
                    let borrowed_left = &mut left_input[..];
                    let borrowed_right = &mut right_input[..];
                    (&mut new_output)
                        .into_iter()
                        .zip(merge(borrowed_left, borrowed_right))
                        .for_each(|(outp, inp)| {
                            *outp = *inp;
                        });
                    SortingTuple {
                        inp_slice: Some(new_output),
                        outp_slice: Some(fuse_slices(left_input, right_input)),
                    }
                }
            },
        );
}
