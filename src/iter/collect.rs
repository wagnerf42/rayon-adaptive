use crate::iter;
use crate::prelude::*;
use crate::traits::BlockedPower;
use rayon::current_num_threads;
use std::iter::repeat;
use std::mem;
pub trait FromAdaptiveBlockedIterator<T>
where
    T: Send,
{
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T, Power = BlockedPower>,
        R: AdaptiveBlockedIteratorRunner<I>;
}

pub trait FromAdaptiveIndexedIterator<T>
where
    T: Send,
{
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIndexedIterator<Item = T>,
        R: AdaptiveIndexedIteratorRunner<I>;
}

//TODO:
// 1) we need to test performances for block sizes
// 2) we still need the fully adaptive algorithm
// 3) extend in parallel ?
impl<T: Send + Sync> FromAdaptiveBlockedIterator<T> for Vec<T> {
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T, Power = BlockedPower>,
        R: AdaptiveBlockedIteratorRunner<I>,
    {
        let capacity = runner.input_len();
        runner
            .partial_fold(
                move || Vec::with_capacity(capacity),
                |mut v, i, limit| {
                    let (todo, remaining) = i.divide_at(limit);
                    v.extend(todo.into_iter()); // optimized extend, yay !
                    (v, remaining)
                },
            )
            .by_blocks(repeat(
                // let's fit in 1mb cache
                1_000_000 * current_num_threads() / mem::size_of::<T>(),
            ))
            .fold(None, |final_v: Option<Vec<T>>, v| {
                if final_v.is_some() {
                    final_v.map(|mut f| {
                        f.extend(v);
                        f
                    })
                } else {
                    Some(v)
                }
            })
            .unwrap_or_else(Vec::new)
    }
}

impl<T: Send + Sync> FromAdaptiveIndexedIterator<T> for Vec<T> {
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIndexedIterator<Item = T>,
        R: AdaptiveIndexedIteratorRunner<I>,
    {
        println!("I know it is indexed");
        let (input, policy) = runner.input_and_policy();
        let output_len = input.base_length();
        let mut output_vector = Vec::with_capacity(output_len);
        unsafe {
            output_vector.set_len(output_len);
        }
        let output_slice: &mut [T] = &mut output_vector;
        println!("start");
        output_slice
            .into_adapt_iter()
            .zip(input)
            .with_policy(policy)
            .for_each(|(out_ref, in_ref)| {
                *out_ref = in_ref;
            });
        println!("end");
        output_vector
    }
}
