//! This crate provides mechanisms for designing adaptive algorithms for rayon.
extern crate rayon_logs as rayon;
//extern crate rayon;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
mod traits;
pub use traits::{Divisible, Mergeable};
mod scheduling;
pub use scheduling::Policy;
mod utils;
pub use utils::fuse_slices;
mod slices;
pub use slices::{EdibleSlice, EdibleSliceMut};
mod algorithms;
pub use algorithms::infix_solvers::*;
pub use algorithms::merge_sort::adaptive_sort;
pub use algorithms::prefix::adaptive_prefix;

/// Execute potentially `oper_a` and `oper_b` in parallel like in a standard join.
/// Then the last closure to finish calls `oper_c` on both results.
pub fn depjoin<A, B, C, RA, RB, RC>(oper_a: A, oper_b: B, oper_c: C) -> RC
where
    A: FnOnce() -> RA + Send,
    B: FnOnce() -> RB + Send,
    C: FnOnce(RA, RB) -> RC + Send,
    RA: Send,
    RB: Send,
    RC: Send,
{
    let done = &AtomicBool::new(false);
    let (sender_a, receiver_b) = channel();
    let (sender_b, receiver_a) = channel();
    let results = rayon::join(
        move || {
            let ra = oper_a();
            let we_are_last = done.swap(true, Ordering::SeqCst);
            if we_are_last {
                let rb = receiver_a.recv().expect("receiving result failed");
                Some(oper_c(ra, rb))
            } else {
                sender_a.send((ra, oper_c)).expect("sending result failed");
                None
            }
        },
        move || {
            let rb = oper_b();
            let we_are_last = done.swap(true, Ordering::SeqCst);
            if we_are_last {
                let (ra, oper_c) = receiver_b.recv().expect("receiving result failed");
                Some(oper_c(ra, rb))
            } else {
                sender_b.send(rb).expect("sending result failed");
                None
            }
        },
    );
    results.0.or(results.1).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let s: u64 = depjoin(
            || (1..100_000).sum(),
            || (1..1000).sum(),
            |sa: u64, sb: u64| sa + sb,
        );
        assert_eq!(s, 5000449500);
        let s: u64 = depjoin(
            || (1..1000).sum(),
            || (1..100_000).sum(),
            |sa: u64, sb: u64| sa + sb,
        );
        assert_eq!(s, 5000449500);
    }
}
