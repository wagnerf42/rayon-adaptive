use prelude::*;
use rayon::current_num_threads;
use std::iter::repeat;
use std::mem;
pub trait FromAdaptiveIterator<T>
where
    T: Send,
{
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T>,
        R: AdaptiveIteratorRunner<I>;
}

//TODO:
// 1) we need two versions, one for exact iterators and the other one
// 2) we need to test performances for block sizes
// 3) we still need the fully adaptive algorithm
// 4) extend in parallel ?
impl<T: Send + Sync> FromAdaptiveIterator<T> for Vec<T> {
    fn from_adapt_iter<I, R>(runner: R) -> Self
    where
        I: AdaptiveIterator<Item = T>,
        R: AdaptiveIteratorRunner<I>,
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
