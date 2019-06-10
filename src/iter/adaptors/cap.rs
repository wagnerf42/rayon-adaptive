//! Cap algorithm to the given number of threads.
//! TODO: this does not work with blocks for now.
//! we need to make a distinction between dividing on the left for sequential iterations
//! and dividing on the right for parallel iterations.
use crate::prelude::*;
use std::ops::Drop;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Switch underlying iterator to adaptive policy (if not specified yet)
/// and cap the number of threads for it's tasks to the given number.
pub struct Cap<I> {
    iterator: I,
    count: Arc<AtomicUsize>,
    limit: usize,
}

impl<I> Drop for Cap<I> {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::Relaxed);
    }
}

// impl<I: ParallelIterator> Divisible for Cap<I> {
//     type Power = I::Power;
//     fn base_length(&self) -> Option<usize> {
//         self.iterator.base_length()
//     }
//     fn divide_at(self, index: usize) -> (Self, Self) {
//         if self.base_length().is_none() {
//             // this is a block operation on an infinite iterator.
//             // authorize it.
//             let (left, right) = self.iterator.divide_at(index);
//             (
//                 Cap {
//                     iterator: left,
//                     count: self.count.clone(),
//                     limit: self.limit,
//                 },
//                 Cap {
//                     iterator: right,
//                     count: self.count,
//                     limit: self.limit,
//                 },
//             )
//         } else {
//             let usage = self.count.fetch_add(1, Ordering::Relaxed);
//             if usage > self.limit {
//                 let (left, right) = self.iterator.divide_at(index);
//                 (
//                     Cap {
//                         iterator: left,
//                         count: self.count.clone(),
//                         limit: self.limit,
//                     },
//                     Cap {
//                         iterator: right,
//                         count: self.count,
//                         limit: self.limit,
//                     },
//                 )
//             } else {
//                 unimplemented!("what is the right way to fail ?")
//             }
//         }
//     }
// }
