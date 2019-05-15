#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::ThreadPoolBuilder;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use std::iter::repeat;
// use std::cmp::min;
// use std::ops::Mul;
// use std::slice::{Iter, IterMut};
//
// // TODO: en profiter pour penser a la taille infinie
//
// trait AdaptiveIterator: Divisible {
//     type SequentialIterator: Iterator;
//     fn iter(self, size: usize) -> (Self, Self::SequentialIterator);
// }
//
// // exemple d'implem manuelle
//
// impl<'a, T: 'a> AdaptiveIterator for &'a [T] {
//     type SequentialIterator = Iter<'a, T>;
//     fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
//         let (now, later) = self.split_at(size);
//         (later, now.into_iter())
//     }
// }
//
//
// impl<'a, T: 'a> AdaptiveIterator for &'a mut [T] {
//     type SequentialIterator = IterMut<'a, T>;
//     fn iter(self, size: usize) -> (Self, Self::SequentialIterator) {
//         let (now, later) = self.divide_at(size);
//         (later, now.into_iter())
//     }
// }
//
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
// // pour le work, on va faire un iterateur sur 0 elements pour tous les blocs sauf le dernier
// // on va communiquer l'input d'un bloc a l'autre a travers un raw pointer comme pour les puissances.
//
// struct Work<I, F> {
//     input: Box<I>,
//     work_function: F,
// }
//
// fn main() {
//     let mut powers = Powers {
//         base: 2,
//         current_output: Box::new(1),
//         size: 10,
//     };
//     let (remaining, first_three) = powers.iter(3);
//     for x in first_three {
//         println!("power: {}", x);
//     }
//     // let's not drop nothing
//     let (nothing, last_powers) = remaining.iter(200);
//     for x in last_powers {
//         println!("next power: {}", x);
//     }
// }
fn main() {
    let v: Vec<u64> = (0..100_000u64).collect();
    let pool = ThreadPoolBuilder::new()
        .build()
        .expect("failed building pool");
    pool.install(|| {
        assert_eq!(
            (v.as_slice())
                .with_policy(Policy::Adaptive(2_000, 20_000))
                .by_blocks(repeat(50_000))
                .max(),
            Some(&99_999)
        )
    });
}
