#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
use std::iter::repeat;
// // exemple d'iterateur pour lequel la division coute qqch
// struct Powers<E: Mul<Output = E> + Clone> {
//     base: E,
//     current_output: Box<E>,
//     size: usize,
// }
//
// impl<E: Mul<Output = E> + Clone> Divisible for Powers<E> {
//     fn base_length(&self) -> usize {
//         self.size
//     }
//     fn divide_at(mut self, index: usize) -> (Self, Self) {
//         let left_size = index;
//         let right_size = self.size - left_size;
//         self.size = left_size;
//         let right = Powers {
//             base: self.base.clone(),
//             current_output: Box::new(power(
//                 (*self.current_output).clone(),
//                 self.base.clone(),
//                 left_size,
//             )),
//             size: right_size,
//         };
//         (self, right)
//     }
//     fn divide(self) -> (Self, Self) {
//         let mid = self.size / 2;
//         self.divide_at(mid)
//     }
// }
//
// fn power<E: Mul<Output = E> + Clone>(value: E, base: E, multiplications_number: usize) -> E {
//     if multiplications_number == 0 {
//         value
//     } else {
//         let tmp = base.clone() * base.clone();
//         if multiplications_number % 2 == 0 {
//             power(value, tmp, multiplications_number / 2)
//         } else {
//             base * power(value, tmp, multiplications_number / 2)
//         }
//     }
// }
//
// struct PowerIterator<E> {
//     base: E,
//     current_output: *mut E,
//     size: usize,
// }
//
// impl<E: Mul<Output = E> + Clone> Iterator for PowerIterator<E> {
//     type Item = E;
//     fn next(&mut self) -> Option<E> {
//         if self.size == 0 {
//             None
//         } else {
//             self.size -= 1;
//             let current_output = unsafe { self.current_output.as_mut() }.unwrap();
//             *current_output = current_output.clone() * self.base.clone();
//             Some(current_output.clone())
//         }
//     }
// }
//
// impl<E: Mul<Output = E> + Clone> AdaptiveIterator for Powers<E> {
//     type SequentialIterator = PowerIterator<E>;
//     fn iter(mut self, size: usize) -> (Self, Self::SequentialIterator) {
//         let raw_pointer = Box::into_raw(self.current_output);
//         let sequential_iterator = PowerIterator {
//             base: self.base.clone(),
//             current_output: raw_pointer,
//             size: min(self.size, size),
//         };
//         self.size = if size > self.size {
//             0
//         } else {
//             self.size - size
//         };
//         self.current_output = unsafe { Box::from_raw(raw_pointer) };
//         (self, sequential_iterator)
//     }
// }
//
fn main() {
    let v: Vec<u64> = (0..100_000u64).collect();
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("failed building pool");
    pool.install(|| {
        assert_eq!(
            v.as_slice().into_par_iter().by_blocks(repeat(50_000)).max(),
            Some(&99_999)
        )
    });
}
