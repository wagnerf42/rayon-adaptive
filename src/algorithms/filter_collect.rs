use {Divisible, DivisibleAtIndex, EdibleSlice, EdibleSliceMut, Policy};
struct FilterWork<'a, T: 'a> {
    input: EdibleSlice<'a, T>,
    output: EdibleSliceMut<'a, T>,
}

// we need to implement it manually to split output at best index.
impl<'a, T: Sync + Send> Divisible for FilterWork<'a, T> {
    fn len(&self) -> usize {
        self.input.len()
    }
    fn split(self) -> (Self, Self) {
        let (left_input, right_input) = self.input.split();
        let remaining_left_size = left_input.len();
        let (left_output, right_output) = self.output.split_at(remaining_left_size);
        (
            FilterWork {
                input: left_input,
                output: left_output,
            },
            FilterWork {
                input: right_input,
                output: right_output,
            },
        )
    }
}

impl<'a, T: Sync + Send> DivisibleAtIndex for FilterWork<'a, T> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left_input, right_input) = self.input.split_at(index);
        let remaining_left_size = left_input.len();
        let (left_output, right_output) = self.output.split_at(remaining_left_size);
        (
            FilterWork {
                input: left_input,
                output: left_output,
            },
            FilterWork {
                input: right_input,
                output: right_output,
            },
        )
    }
}

/// Filter given slice by given function and collect into vector.
pub fn filter_collect<T, F>(slice: &[T], filter: F, policy: Policy) -> Vec<T>
where
    T: Send + Sync + Copy,
    F: Fn(&&T) -> bool + Sync,
{
    let size = slice.len();
    let mut uninitialized_output = Vec::with_capacity(size);
    unsafe {
        uninitialized_output.set_len(size);
    }
    let used = {
        let input = FilterWork {
            input: EdibleSlice::new(slice),
            output: EdibleSliceMut::new(uninitialized_output.as_mut_slice()),
        };
        let final_output = input
            .work(|mut slices, limit| {
                for (i, o) in slices
                    .input
                    .iter()
                    .take(limit)
                    .filter(|e| filter(e))
                    .zip(slices.output.iter_mut())
                {
                    *o = *i;
                }
                slices
            }).map(|slices| slices.output)
            .fold_with_blocks(
                None,
                |potential_left_slice: Option<EdibleSliceMut<T>>, right_slice| {
                    if let Some(left_slice) = potential_left_slice {
                        Some(left_slice.fuse(right_slice))
                    } else {
                        Some(right_slice)
                    }
                },
                1_000_000,
                policy,
            ).unwrap();
        slice.len() - final_output.len()
    };
    unsafe {
        uninitialized_output.set_len(used);
    }
    uninitialized_output
}
