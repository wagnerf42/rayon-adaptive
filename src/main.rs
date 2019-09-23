use rayon_adaptive::prelude::*;
// use rayon_adaptive::successors;
//
// // schedulers
//
// fn find_first_join<
//     I: FiniteParallelIterator + Divisible,
//     P: Fn(&I::Item) -> bool + Clone + Sync,
// >(
//     mut iter: I,
//     predicate: P,
// ) -> Option<I::Item> {
//     if iter.is_divisible() {
//         let (left, right) = iter.divide();
//         let (left_answer, right_answer) = rayon::join(
//             || find_first_join(left, predicate.clone()),
//             || find_first_join(right, predicate.clone()),
//         );
//         left_answer.or(right_answer)
//     } else {
//         iter.sequential_borrow_on_left_for(iter.len())
//             .find(predicate)
//     }
// }
//
// fn find_first_extract<I, P>(mut input: I, predicate: P) -> Option<I::Item>
// where
//     I: ParallelIterator,
//     P: Fn(&<I as ItemProducer>::Item) -> bool + Sync,
// {
//     let mut found = None;
//     let mut current_size = 1;
//     while found.is_none() {
//         let iter = input.borrow_on_left_for(current_size);
//         found = find_first_join(iter, &predicate);
//         current_size *= 2;
//     }
//     found
// }
//
// fn main() {
//     let s = successors(
//         2u32,
//         |i: u32| i + 2u32,
//         |i: u32, n: usize| i + (n as u32) * 2,
//     );
//     assert_eq!(find_first_extract(s, |&e| e % 100 == 0), Some(100));
//
//     eprintln!(
//         "{}",
//         (0u32..1_000)
//             .into_par_iter()
//             //            .filter(|&i| i % 2 == 0)
//             .map(|i| 2 * i)
//             .take(5)
//             //.iterator_fold(|i| i.sum::<u32>()) // TODO: this is ICE
//             //            .with_join_policy(10)
//             //            .with_rayon_policy()
//             //            .even_levels()
//             .reduce(|| 0, |a, b| a + b)
//     );
//
//     let mut v = vec![1, 2, 3];
//     v.as_mut_slice()
//         .into_par_iter()
//         .zip(0..2)
//         .for_each(|(a, b)| *a = b);
//     eprintln!("v: {:?}", v);
//
//     assert_eq!(
//         find_first_extract((1u32..).into_par_iter(), |&x| x == 1000),
//         Some(1000)
//     );
// }
//
fn main() {
    let r = (0u32..10).into_par_iter();
    eprintln!("r: {:?}", r);
    assert_eq!((0u32..10).into_par_iter().sum::<u32>(), 45);
}
