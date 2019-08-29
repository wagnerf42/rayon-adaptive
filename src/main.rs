mod join;
mod local;
mod map;
pub mod prelude;
mod range;
mod successors;
use crate::prelude::*;
use range::ParRange;
use successors::ParSuccessors;

// TODO: will I need two extractible traits ?
// TODO: where goes the power ?
// TODO: what about even_levels
// TODO: by_blocks

// schedulers

fn find_first_join<
    I: FiniteParallelIterator,
    P: Fn(&<I as ItemProducer>::Item) -> bool + Clone + Sync,
>(
    iter: I,
    predicate: P,
) -> Option<I::Item> {
    if iter.is_divisible() {
        let (left, right) = iter.divide();
        let (left_answer, right_answer) = rayon::join(
            || find_first_join(left, predicate.clone()),
            || find_first_join(right, predicate.clone()),
        );
        left_answer.or(right_answer)
    } else {
        iter.to_sequential().find(predicate)
    }
}

fn find_first_extract<I, P>(mut input: I, predicate: P) -> Option<<I as ItemProducer>::Item>
where
    I: ParallelIterator,
    P: Fn(&<I as ItemProducer>::Item) -> bool + Sync,
{
    let mut found = None;
    let mut current_size = 1;
    while found.is_none() {
        let iter = input.borrow_on_left_for(current_size);
        found = find_first_join(iter, &predicate);
        current_size *= 2;
    }
    found
}

fn integer_sum<I: FiniteParallelIterator + ItemProducer<Item = u32>>(iter: I) -> u32 {
    if iter.is_divisible() {
        let (left, right) = iter.divide();
        let (left_answer, right_answer) = rayon::join(|| integer_sum(left), || integer_sum(right));
        left_answer + right_answer
    } else {
        iter.to_sequential().sum()
    }
}

fn main() {
    let s = ParSuccessors {
        next: 2u32,
        succ: |i: u32| i + 2u32,
        skip_op: |i: u32, n: usize| i + (n as u32) * 2,
    };
    assert_eq!(find_first_extract(s, |&e| e % 100 == 0), Some(100));

    eprintln!(
        "{}",
        integer_sum(
            ParRange { range: 0..1_000 }
                .map(|i| 2 * i)
                .with_join_policy(10)
                .with_rayon_policy()
        )
    );
}
