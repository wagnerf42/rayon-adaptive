//! Adaptive prefix algorithm.
//! No macro blocks.
use std::collections::LinkedList;
use {Divisible, EdibleSliceMut, Policy};

pub fn adaptive_prefix<T, O>(v: &mut [T], op: O, policy: Policy)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    let input = EdibleSliceMut::new(v);
    let mut list = input.work(
        |mut slice, limit| {
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
        },
        |slice| {
            let mut list = LinkedList::new();
            list.push_back(slice.slice());
            list
        },
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
        input.work(
            |mut s, limit| {
                for e in s.iter_mut().take(limit) {
                    *e = op(e, &increment)
                }
                s
            },
            |_| (),
            |_, _| (),
            Policy::Adaptive(10000),
        );
    }
    slice.last().cloned().unwrap()
}
