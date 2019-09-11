mod cloned;
mod dislocated;
mod even_levels;
mod filter;
mod iterator_fold;
mod join;
mod local;
mod map;
pub mod prelude;
mod range;
mod scheduler;
mod slice;
mod successors;
use crate::prelude::*;
use range::ParRange;
use successors::ParSuccessors;

// TODO: by_blocks -> we need a method giving the blocks sizes
//
//
#[macro_use]
mod private;
pub(crate) use private_try::Try;

/// We hide the `Try` trait in a private module, as it's only meant to be a
/// stable clone of the standard library's `Try` trait, as yet unstable.
/// this snippet is taken directly from rayon.
mod private_try {
    /// Clone of `std::ops::Try`.
    ///
    /// Implementing this trait is not permitted outside of `rayon`.
    pub trait Try {
        private_decl! {}

        type Ok;
        type Error;
        fn into_result(self) -> Result<Self::Ok, Self::Error>;
        fn from_ok(v: Self::Ok) -> Self;
        fn from_error(v: Self::Error) -> Self;
    }

    impl<T> Try for Option<T> {
        private_impl! {}

        type Ok = T;
        type Error = ();

        fn into_result(self) -> Result<T, ()> {
            self.ok_or(())
        }
        fn from_ok(v: T) -> Self {
            Some(v)
        }
        fn from_error(_: ()) -> Self {
            None
        }
    }

    impl<T, E> Try for Result<T, E> {
        private_impl! {}

        type Ok = T;
        type Error = E;

        fn into_result(self) -> Result<T, E> {
            self
        }
        fn from_ok(v: T) -> Self {
            Ok(v)
        }
        fn from_error(v: E) -> Self {
            Err(v)
        }
    }
}

// schedulers

fn find_first_join<
    I: FiniteParallelIterator + Divisible,
    P: Fn(&I::Item) -> bool + Clone + Sync,
>(
    mut iter: I,
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
        iter.sequential_borrow_on_left_for(iter.len())
            .find(predicate)
    }
}

fn find_first_extract<I, P>(mut input: I, predicate: P) -> Option<I::Item>
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

fn main() {
    let s = ParSuccessors {
        next: 2u32,
        succ: |i: u32| i + 2u32,
        skip_op: |i: u32, n: usize| i + (n as u32) * 2,
    };
    assert_eq!(find_first_extract(s, |&e| e % 100 == 0), Some(100));

    eprintln!(
        "{}",
        ParRange { range: 0..1_000 }
            .filter(|&i| i % 2 == 0)
            .map(|i| 2 * i)
            //.iterator_fold(|i| i.sum::<u32>()) // TODO: this is ICE
            .with_join_policy(10)
            .with_rayon_policy()
            .even_levels()
            .reduce(|| 0, |a, b| a + b)
    );
}
