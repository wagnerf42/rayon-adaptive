//! Adaptive prefix algorithm.
//! No macro blocks.
use crate::{prelude::*, BlockedPower, EdibleSliceMut};
use rayon::scope;
use std::iter::repeat;

/// Run adaptive prefix algortihm on given slice.
/// Each element is replaced by folding with op since beginning of the slice.
/// It requires an associative operation.
///
/// # Example
///
/// ```
/// use rayon_adaptive::adaptive_prefix;
/// let mut v = vec![1u32; 100_000];
/// adaptive_prefix(&mut v, |e1, e2| e1 + e2);
/// let count: Vec<u32> = (1..=100_000).collect();
/// assert_eq!(v, count);
/// ```
pub fn adaptive_prefix<T, O>(v: &mut [T], op: O)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    let input = EdibleSliceMut::new(v);
    input
        .work(|mut slice, limit| {
            let c = {
                let mut elements = slice.iter_mut().take(limit);
                let mut c = elements.next().unwrap().clone();
                for e in elements {
                    *e = op(e, &c);
                    c = e.clone();
                }
                c
            };
            // pre-update next one
            if let Some(e) = slice.peek() {
                *e = op(e, &c);
            }
            slice
        })
        .map(|slice| slice.slice())
        .into_iter()
        .fold(
            None,
            |potential_previous_slice: Option<&mut [T]>, current_slice| {
                if let Some(previous_slice) = potential_previous_slice {
                    update(current_slice, previous_slice.last().cloned().unwrap(), &op);
                }
                Some(current_slice)
            },
        );
}

fn update<T, O>(slice: &mut [T], increment: T, op: &O)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    slice.into_adapt_iter().for_each(|e| *e = op(e, &increment))
}

// now the fully adaptive version

struct PrefixSlice<'a, T: 'a + Send + Sync> {
    slice: &'a mut [T],
    index: usize,
}

impl<'a, T: 'a + Send + Sync> Divisible for PrefixSlice<'a, T> {
    type Power = BlockedPower;
    fn base_length(&self) -> usize {
        self.slice.len() - self.index
    }
    fn divide(self) -> (Self, Self) {
        let middle = self.base_length() / 2;
        let (left, right) = self.slice.split_at_mut(self.index + middle);
        (
            PrefixSlice {
                slice: left,
                index: self.index,
            },
            PrefixSlice {
                slice: right,
                index: 0,
            },
        )
    }
}

impl<'a, T: 'a + Send + Sync> DivisibleIntoBlocks for PrefixSlice<'a, T> {
    fn divide_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.slice.split_at_mut(self.index + index);
        (
            PrefixSlice {
                slice: left,
                index: self.index,
            },
            PrefixSlice {
                slice: right,
                index: 0,
            },
        )
    }
}

pub fn fully_adaptive_prefix<T, O>(input_vector: &mut [T], op: O)
where
    T: Send + Sync + Copy,
    O: Fn(&T, &T) -> T + Sync + Send + Copy,
{
    let first_value = input_vector.first().cloned().unwrap();
    let length = input_vector.len();
    let input = PrefixSlice {
        slice: &mut input_vector[1..],
        index: 0,
    };

    scope(|s| {
        input
            .by_blocks(repeat(length / 10))
            .work(|mut prefix_slice, limit| {
                if prefix_slice.index == 0 {
                    let previous_value = prefix_slice.slice.first().cloned().unwrap();
                    prefix_slice.slice[1..(prefix_slice.index + limit)]
                        .iter_mut()
                        .fold(previous_value, |previous_value, e| {
                            *e = op(&previous_value, e);
                            *e
                        });
                } else {
                    let previous_value = prefix_slice
                        .slice
                        .get(prefix_slice.index - 1)
                        .cloned()
                        .unwrap();
                    prefix_slice.slice[prefix_slice.index..(prefix_slice.index + limit)]
                        .iter_mut()
                        .fold(previous_value, |previous_value, e| {
                            *e = op(&previous_value, e);
                            *e
                        });
                }
                prefix_slice.index += limit;
                prefix_slice
            })
            .map(|s| s.slice)
            .helping_cutting_fold(
                first_value,
                |last_elem_prev_slice, prefix_slice| {
                    prefix_slice
                        .slice
                        .iter_mut()
                        .fold(last_elem_prev_slice, |c, e| {
                            *e = op(&c, e);
                            *e
                        })
                },
                |last_num, slice| {
                    if let Some(last_slice_num) = slice.last().cloned() {
                        s.spawn(move |_| {
                            slice.into_adapt_iter().for_each(|e| *e = op(&last_num, e))
                        });
                        op(&last_num, &last_slice_num)
                    } else {
                        last_num
                    }
                },
            )
    });
}
