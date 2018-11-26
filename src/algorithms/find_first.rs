use std::cmp::min;
use {Divisible, EdibleSlice};

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
    let input = EdibleSlice::new(v);
    input
        .fold(
            || None,
            |found, mut slice, limit| {
                (
                    found.or_else(|| slice.iter().take(limit).find(|e| f(e)).cloned()),
                    slice,
                )
            },
        ).by_blocks(powers(base_size))
        .filter_map(|o| o)
        .next()
}
