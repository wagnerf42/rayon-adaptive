extern crate rand;
extern crate rayon_adaptive;
extern crate rayon_logs;
use rand::random;
use rayon_adaptive::{AdaptiveWork, Policy};
use rayon_logs::ThreadPoolBuilder;
use std::collections::LinkedList;

struct PrefixWork<'a> {
    i: usize, // we worked until here
    slice: &'a mut [u32],
}

impl<'a> AdaptiveWork for PrefixWork<'a> {
    type Output = LinkedList<&'a mut [u32]>;
    fn work(&mut self, limit: usize) {
        let mut c = if self.i == 0 {
            0
        } else {
            self.slice[self.i - 1]
        };
        for e in self.slice[self.i..].iter_mut().take(limit) {
            *e += c;
            c = *e;
        }
        self.i += limit;
    }
    fn output(self) -> Self::Output {
        let mut list = LinkedList::new();
        list.push_back(self.slice);
        list
    }
    fn remaining_length(&self) -> usize {
        self.slice.len() - self.i
    }
    fn split(self) -> (Self, Self) {
        let mid = self.i + self.remaining_length() / 2;
        let (my_part, his_part) = self.slice.split_at_mut(mid);
        (
            PrefixWork {
                i: self.i,
                slice: my_part,
            },
            PrefixWork {
                i: 0,
                slice: his_part,
            },
        )
    }
}

fn prefix(v: &mut [u32], policy: Policy) {
    let input = PrefixWork { i: 0, slice: v };
    let mut list = input.schedule(policy);
    let first = list.pop_front().unwrap();
    let mut current_value = first.last().cloned().unwrap();
    for slice in list.iter_mut() {
        current_value = update(slice, current_value);
    }
}

struct UpdateWork<'a> {
    i: usize,
    slice: &'a mut [u32],
    increment: u32,
}

impl<'a> AdaptiveWork for UpdateWork<'a> {
    type Output = ();
    fn work(&mut self, limit: usize) {
        for e in self.slice.iter_mut().take(limit) {
            *e += self.increment;
        }
        self.i += limit;
    }
    fn output(self) -> Self::Output {
        ()
    }
    fn remaining_length(&self) -> usize {
        self.slice.len() - self.i
    }
    fn split(self) -> (Self, Self) {
        //TODO: think again at all this duplicated code
        let mid = self.i + self.remaining_length() / 2;
        let (my_part, his_part) = self.slice.split_at_mut(mid);
        (
            UpdateWork {
                i: self.i,
                slice: my_part,
                increment: self.increment,
            },
            UpdateWork {
                i: 0,
                slice: his_part,
                increment: self.increment,
            },
        )
    }
}

fn update(slice: &mut [u32], increment: u32) -> u32 {
    {
        let input = UpdateWork {
            i: 0,
            slice,
            increment,
        };
        input.schedule(Policy::JoinContext(10000));
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
        prefix(&mut v, Policy::JoinContext(10000));
    }).1;
    log.save_svg("prefix.svg").expect("saving svg failed");
    assert_eq!(v, answer);
}
