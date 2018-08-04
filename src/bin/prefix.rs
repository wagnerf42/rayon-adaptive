extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{fuse_slices, Divisible, Mergeable, Policy};
use rayon_logs::ThreadPoolBuilder;
use std::collections::LinkedList;

struct InputSlice<'a> {
    slice: &'a mut [u32],
    partial: bool, // if true we will not compute the real result
}

impl<'a> Divisible for InputSlice<'a> {
    fn len(&self) -> usize {
        self.slice.len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.slice.len() / 2;
        let (left, right) = self.slice.split_at_mut(mid);
        (
            InputSlice {
                slice: left,
                partial: self.partial,
            },
            InputSlice {
                slice: right,
                partial: true,
            },
        )
    }
}

#[derive(Debug)]
struct OutputSlice<'a> {
    slices: LinkedList<&'a mut [u32]>,
    partial: bool, // we need an update from DIRECT predecessor
}

impl<'a> Mergeable for OutputSlice<'a> {
    fn fuse(self, other: Self) -> Self {
        let mut left = self;
        let mut right = other;
        if right.partial {
            left.slices.append(&mut right.slices);
            OutputSlice {
                slices: left.slices,
                partial: left.partial,
            }
        } else {
            let left_slice = left.slices.pop_back().unwrap();
            let right_slice = right.slices.pop_back().unwrap();
            assert!(right.slices.is_empty());
            let slice = fuse_slices(left_slice, right_slice);
            left.slices.push_back(slice);
            OutputSlice {
                slices: left.slices,
                partial: left.partial,
            }
        }
    }
}

//TODO: think again
//all this would be easier with two kind of outputs
//a local output which is fused differently from a global (stolen) output
fn prefix(v: &mut [u32], policy: Policy) {
    let input = InputSlice {
        slice: v,
        partial: false,
    };
    let list = input.work(
        |input, limit| {
            let last_value = {
                let mut elements = input.slice.iter_mut().take(limit);
                let mut acc = elements.next().cloned().unwrap();
                for e in elements {
                    *e += acc;
                    acc = *e;
                }
                acc
            };
            if input.slice.len() > limit {
                input.slice[limit] += last_value;
                let (computed_slice, remaining_slice) = input.slice.split_at_mut(limit);
                let mut l = LinkedList::new();
                l.push_back(computed_slice);
                (
                    Some(InputSlice {
                        slice: remaining_slice,
                        partial: false,
                    }),
                    OutputSlice {
                        slices: l,
                        partial: input.partial,
                    },
                )
            } else {
                let mut l = LinkedList::new();
                l.push_back(input.slice);
                (
                    None,
                    OutputSlice {
                        slices: l,
                        partial: input.partial,
                    },
                )
            }
        },
        policy,
    );
    println!("{:?}", list);
    unimplemented!()
}

fn main() {
    let mut v: Vec<u32> = (0..10).map(|_| random::<u32>() % 3).collect();
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
        prefix(&mut v, Policy::Adaptive(2000));
    }).1;
    log.save_svg("prefix.svg").expect("saving svg failed");
    assert_eq!(v, answer);
}
