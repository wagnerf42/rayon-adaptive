use crate::prelude::*;
use crate::traits::BlockedPower;
use rayon::current_num_threads;
use std::iter::repeat;
use std::mem;
pub trait FromAdaptiveBlockedIterator<T>
where
    T: Send,
{
    fn from_adapt_iter<I, R, S>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T, Power = BlockedPower>,
        R: AdaptiveBlockedIteratorRunner<I, S>,
        S: Iterator<Item = usize>;
}

pub trait FromAdaptiveIndexedIterator<T>
where
    T: Send,
{
    fn from_adapt_iter<I, R, S>(runner: R) -> Self
    where
        I: AdaptiveIndexedIterator<Item = T>,
        R: AdaptiveIndexedIteratorRunner<I, S>,
        S: Iterator<Item = usize>;
}

//TODO:
// 1) we need to test performances for block sizes
// 2) we still need the fully adaptive algorithm
// 3) extend in parallel ?
impl<T: Send + Sync> FromAdaptiveBlockedIterator<T> for Vec<T> {
    fn from_adapt_iter<I, R, S>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T, Power = BlockedPower>,
        R: AdaptiveBlockedIteratorRunner<I, S>,
        S: Iterator<Item = usize>,
    {
        let (input, policy, sizes) = runner.input_policy_sizes();
        let capacity = input.base_length();
        input
            .with_policy(policy)
            .by_blocks(sizes.chain(repeat(
                // let's fit in 1mb cache
                1_000_000 * current_num_threads() / mem::size_of::<T>(),
            )))
            .partial_fold(
                move || Vec::with_capacity(capacity),
                |mut v, i, limit| {
                    let (todo, remaining) = i.divide_at(limit);
                    v.extend(todo.into_iter()); // optimized extend, yay !
                    (v, remaining)
                },
            )
            .into_iter()
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
    fn from_adapt_iter<I, R, S: Iterator<Item = usize>>(runner: R) -> Self
    where
        I: AdaptiveIndexedIterator<Item = T>,
        R: AdaptiveIndexedIteratorRunner<I, S>,
    {
        let (input, policy, sizes) = runner.input_policy_sizes();
        let output_len = input.base_length();
        let mut output_vector = Vec::with_capacity(output_len);
        unsafe {
            output_vector.set_len(output_len);
        }
        let output_slice: &mut [T] = &mut output_vector;
        output_slice
            .into_adapt_iter()
            .zip(input)
            .with_policy(policy)
            .by_blocks(sizes)
            .for_each(|(out_ref, in_ref)| unsafe { std::ptr::write(out_ref, in_ref) });
        output_vector
    }
}
