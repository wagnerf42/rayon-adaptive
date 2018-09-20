//! Adaptive prefix algorithm.
//! No genericity yet and no macro blocks.
use std::collections::LinkedList;
use {Divisible, EdibleSliceMut, Policy};

pub fn adaptive_prefix(v: &mut [u32], policy: Policy) {
    let input = EdibleSliceMut::new(v);
    let mut list = input.work(
        |slice, limit| {
            let c = {
                let mut elements = slice.iter_mut().take(limit);
                let mut c = *elements.next().unwrap();
                for e in elements {
                    *e += c;
                    c = *e;
                }
                c
            };
            // pre-update next one
            if let Some(e) = slice.peek() {
                *e += c;
            }
        },
        |slice| {
            let mut list = LinkedList::new();
            list.push_back(slice.slice());
            list
        },
        policy,
    );

    let first = list.pop_front().unwrap();
    let mut current_value = first.last().cloned().unwrap();
    for slice in list.iter_mut() {
        current_value = update(slice, current_value);
    }
}

fn update(slice: &mut [u32], increment: u32) -> u32 {
    {
        let input = EdibleSliceMut::new(slice);
        input.work(
            |s, limit| {
                for e in s.iter_mut().take(limit) {
                    *e += increment
                }
            },
            |_| (),
            Policy::Adaptive(10000),
        );
    }
    slice.last().cloned().unwrap()
}
