extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{Divisible, EdibleSliceMut, Policy};
use rayon_logs::ThreadPoolBuilder;
use std::collections::LinkedList;

fn prefix(v: &mut [u32], policy: Policy) {
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

fn main() {
    let mut v: Vec<u32> = (0..1_000_000).map(|_| random::<u32>() % 3).collect();
    let answer: Vec<u32> = v.iter()
        .scan(0, |acc, x| {
            *acc += *x;
            Some(*acc)
        })
        .collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(2)
        .build()
        .expect("pool creation failed");
    let log = pool.install(|| {
        prefix(&mut v, Policy::Adaptive(10000));
    }).1;
    log.save_svg("prefix.svg").expect("saving svg failed");
    assert_eq!(v, answer);
}
