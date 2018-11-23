//! Fully Adaptive prefix algorithm.
extern crate rayon_adaptive;
// use rayon_adaptive::{fully_adaptive_schedule, Divisible, EdibleSliceMut, KeepLeft, Policy};
//
// fn adaptive_prefix<T, O>(v: &mut [T], op: O)
// where
//     T: Send + Sync + Clone,
//     O: Fn(&T, &T) -> T + Sync,
// {
//     let input: (EdibleSliceMut<T>, KeepLeft<Option<T>>) = (EdibleSliceMut::new(v), KeepLeft(None));
//     fully_adaptive_schedule(
//         input,
//         &|(mut slice, possible_previous_value): (EdibleSliceMut<T>, KeepLeft<Option<T>>),
//           limit: usize|
//          -> (EdibleSliceMut<T>, KeepLeft<Option<T>>) {
//             let c = {
//                 let mut elements = slice.iter_mut().take(limit);
//                 let mut c = if let Some(previous_value) = possible_previous_value.0 {
//                     previous_value
//                 } else {
//                     elements.next().cloned().unwrap()
//                 };
//                 for e in elements {
//                     *e = op(e, &c);
//                     c = e.clone();
//                 }
//                 c
//             };
//             (slice, KeepLeft(Some(c)))
//         },
//         &|left: (EdibleSliceMut<T>, KeepLeft<Option<T>>),
//           achieved_right: (EdibleSliceMut<T>, KeepLeft<Option<T>>),
//           remaining_right: (EdibleSliceMut<T>, KeepLeft<Option<T>>)| {
//             let start_value_for_remaining =
//                 op(left.1.as_ref().unwrap(), achieved_right.1.as_ref().unwrap());
//             update(achieved_right.0.slice(), ((left.1).0).unwrap(), &op);
//             (remaining_right.0, KeepLeft(Some(start_value_for_remaining)))
//         },
//     );
// }
//
// fn update<T, O>(slice: &mut [T], increment: T, op: &O)
// where
//     T: Send + Sync + Clone,
//     O: Fn(&T, &T) -> T + Sync,
// {
//     {
//         let input = EdibleSliceMut::new(slice);
//         input.for_each(
//             |mut s, limit| {
//                 for e in s.iter_mut().take(limit) {
//                     *e = op(e, &increment)
//                 }
//                 s
//             },
//             Policy::Adaptive(1000),
//         );
//     }
// }

fn main() {
    unimplemented!()
    //    let mut v = vec![1u32; 100_000];
    //    adaptive_prefix(&mut v, |e1, e2| e1 + e2);
    //    let count: Vec<u32> = (1..=100_000).collect();
    //    assert_eq!(v, count);
}
