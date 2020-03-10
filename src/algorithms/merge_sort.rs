use crate::prelude::*;
/// Fuse contiguous slices together back into one.
/// This panics if slices are not contiguous.
fn fuse_slices<'a: 'c, 'b: 'c, 'c, T: 'a + 'b>(s1: &'a mut [T], s2: &'b mut [T]) -> &'c mut [T] {
    let ptr1 = s1.as_mut_ptr();
    unsafe {
        assert_eq!(ptr1.add(s1.len()) as *const T, s2.as_ptr());
        std::slice::from_raw_parts_mut(ptr1, s1.len() + s2.len())
    }
}

#[test]
fn test_stability() {
    #[derive(Copy, Clone)]
    struct OpaqueTuple {
        first: u64,
        second: u64,
    }
    unsafe impl Send for OpaqueTuple {}
    unsafe impl Sync for OpaqueTuple {}
    impl PartialEq for OpaqueTuple {
        fn eq(&self, other: &Self) -> bool {
            self.first == other.first
        }
    }
    impl Eq for OpaqueTuple {}
    impl PartialOrd for OpaqueTuple {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.first.cmp(&other.first))
        }
    }
    impl Ord for OpaqueTuple {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.first.cmp(&other.first)
        }
    }
    for len in (2..10).chain(100..110).chain(10_000..10_010) {
        let mut v: Vec<_> = (0u64..len)
            .map(|index| OpaqueTuple {
                first: 2,
                second: index,
            })
            .collect();
        merge_sort_adaptive(&mut v);
        &v.windows(2).for_each(|slice_of_tuples| {
            assert!(slice_of_tuples[0].second < slice_of_tuples[1].second);
        });
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
///    .num_threads(4)
///    .build_global()
///    .expect("pool build failed");
///merge_sort_adaptive(&mut input);
///assert_eq!(input,solution);
///```
pub fn merge_sort_adaptive<'a, T: 'a + Send + Sync + Ord + Copy>(input: &'a mut [T]) {
    let problem_size = input.base_length();
    let mut copy_vector: Vec<T> = Vec::with_capacity(input.len());
    unsafe {
        copy_vector.set_len(input.len());
    }
    let to_sort = (input, copy_vector.as_mut_slice());

    to_sort
        .wrap_iter()
        .map(|s| {
            s.0.sort();
            s
        })
        .with_rayon_policy()
        .with_join_policy(problem_size / rayon::current_num_threads())
        .even_levels()
        .non_adaptive_iter()
        .reduce_with(|(left_input, left_output), (right_input, right_output)| {
            let new_output = fuse_slices(left_output, right_output);
            left_input
                .par_iter()
                .merge(right_input.par_iter())
                .directional_zip(new_output.par_iter_mut())
                .for_each(|(sorted, placeholder)| *placeholder = *sorted);
            (new_output, fuse_slices(left_input, right_input))
        });
}
