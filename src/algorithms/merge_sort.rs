use crate::prelude::*;
use itertools::merge;

/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
fn fuse_slices<'a: 'c, 'b: 'c, 'c, T: 'a + 'b>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

pub fn merge_sort_adaptive<'a, T: 'a + Send + Sync + Ord + Copy>(
    input: &'a mut [T],
    threshold: usize,
) {
    let mut copy_vector: Vec<T> = Vec::with_capacity(input.len());
    unsafe {
        copy_vector.set_len(input.len());
    }
    let to_sort = (input, copy_vector.as_mut_slice());

    to_sort
        .wrap()
        .non_adaptive_iter()
        .map(|s| {
            s.0.sort();
            s
        })
        .with_join_policy(threshold)
        .even_levels()
        .reduce_with(|(left_input, left_output), (right_input, right_output)| {
            let new_output = fuse_slices(left_output, right_output);
            new_output
                .into_iter()
                .zip(merge(&mut left_input[..], &mut right_input[..]))
                .for_each(|(outp, inp)| {
                    *outp = *inp;
                });
            (new_output, fuse_slices(left_input, right_input))
        });
}
