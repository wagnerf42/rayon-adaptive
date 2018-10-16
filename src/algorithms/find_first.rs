use std::cmp::min;
use std::mem::replace;
use {Divisible, EdibleSlice, KeepLeft};

fn powers(starting_value: usize) -> impl Iterator<Item = usize> {
    (0..).scan(starting_value, |state, _| {
        *state *= 2;
        Some(*state)
    })
}

/// Return first element for which f returns true.
pub fn find_first<T, F>(v: &[T], f: F) -> Option<T>
where
    T: Sync + Send + Copy,
    F: Fn(&&T) -> bool + Sync,
{
    let base_size = min((v.len() as f64).log(2.0).ceil() as usize, v.len());
    let input = (EdibleSlice::new(v), KeepLeft(None));
    input
        .work(|mut slice, limit| {
            if slice.1.is_none() {
                replace(
                    &mut (slice.1).0,
                    slice.0.iter().take(limit).find(|e| f(e)).cloned(),
                );
            }
            slice
        }).map(|slice| (slice.1).0)
        .by_blocks(powers(base_size))
        .filter_map(|o| o)
        .next()
}
