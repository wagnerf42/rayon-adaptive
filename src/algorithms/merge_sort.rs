use crate::prelude::*;
use itertools::merge;
#[cfg(feature = "logs")]
use rayon_logs::subgraph;

/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
fn fuse_slices<'a: 'c, 'b: 'c, 'c, T: 'a + 'b>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

fn raw_merge<T: Ord + Send + Sync + Copy>(left: &[T], right: &[T], output: &mut [T]) {
    let left_len = left.len();
    let right_len = right.len();
    let output_len = output.len();
    debug_assert_eq!(output_len, left_len + right_len);
    if left.last() <= right.first() {
        output[..left_len].copy_from_slice(left);
        output[left_len..].copy_from_slice(right);
    } else if left.first() > right.last() {
        output[..right_len].copy_from_slice(right);
        output[right_len..].copy_from_slice(left);
    } else {
        let mut left_index = 0;
        let mut right_index = 0;
        for output_index in 0..output_len {
            let output_ref = unsafe { output.get_unchecked_mut(output_index) };
            unsafe {
                if left.get_unchecked(left_index) <= right.get_unchecked(right_index) {
                    *output_ref = *left.get_unchecked(left_index);
                    left_index += 1;
                    if left_index == left_len {
                        output[output_index + 1..].copy_from_slice(&right[right_index..]);
                        break;
                    }
                } else {
                    *output_ref = *right.get_unchecked(right_index);
                    right_index += 1;
                    if right_index == right_len {
                        output[output_index + 1..].copy_from_slice(&left[left_index..]);
                        break;
                    }
                }
            }
        }
    }
}

///Example:
///```
///use rand::thread_rng;
///use rayon_adaptive::merge_sort_adaptive;
///use rand::seq::SliceRandom;
///let mut input = (1..25_000_001u32).collect::<Vec<u32>>();
///input.shuffle(&mut thread_rng());
///let solution = (1..25_000_001u32).collect::<Vec<u32>>();
///rayon::ThreadPoolBuilder::new()
///    .num_threads(8)
///    .build_global()
///    .expect("pool build failed");
///merge_sort_adaptive(&mut input, 25_000_001/8);
///assert_eq!(input,solution);
///```
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
        .with_rayon_policy()
        .with_join_policy(threshold) //The constant here should be number of threads + 1
        .even_levels()
        .reduce_with(|(left_input, left_output), (right_input, right_output)| {
            let new_output = fuse_slices(left_output, right_output);
            raw_merge(&left_input[..], &right_input[..], new_output);
            (new_output, fuse_slices(left_input, right_input))
        });
}
