//! Adaptive prefix algorithm.
//! No macro blocks.
use std::collections::LinkedList;
use {Divisible, EdibleSliceMut, Policy};

/// Run adaptive prefix algortihm on given slice.
/// Each element is replaced by folding with op since beginning of the slice.
/// It requires an associative operation.
///
/// # Example
///
/// ```
/// use rayon_adaptive::{adaptive_prefix, Policy};
/// let mut v = vec![1u32; 100_000];
/// adaptive_prefix(&mut v, |e1, e2| e1 + e2, Policy::Adaptive(1000));
/// let count: Vec<u32> = (1..=100_000).collect();
/// assert_eq!(v, count);
/// ```
pub fn adaptive_prefix<T, O>(v: &mut [T], op: O, policy: Policy)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    let input = EdibleSliceMut::new(v);
    let mut list = input
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
        }).map(|slice| {
            let mut list = LinkedList::new();
            list.push_back(slice.slice());
            list
        }).reduce(
            |mut left, mut right| {
                left.append(&mut right);
                left
            },
            policy,
        );

    let first = list.pop_front().unwrap();
    let mut current_value = first.last().cloned().unwrap();
    for slice in list.iter_mut() {
        current_value = update(slice, current_value, &op);
    }
}

fn update<T, O>(slice: &mut [T], increment: T, op: &O) -> T
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    {
        let input = EdibleSliceMut::new(slice);
        input.for_each(
            |mut s, limit| {
                for e in s.iter_mut().take(limit) {
                    *e = op(e, &increment)
                }
                s
            },
            Policy::Adaptive(1000),
        );
    }
    slice.last().cloned().unwrap()
}
