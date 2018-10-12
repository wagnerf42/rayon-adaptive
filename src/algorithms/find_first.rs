use std::mem::replace;
use {Divisible, EdibleSlice, KeepLeft, Policy};

fn powers_of_two() -> impl Iterator<Item = usize> {
    (0..).scan(1, |state, _| {
        *state *= 2;
        Some(*state)
    })
}

/// Return first element for which f returns true.
pub fn find_first<T, F>(v: &[T], f: F, policy: Policy) -> Option<T>
where
    T: Sync + Send + Copy,
    F: Fn(&&T) -> bool + Sync,
{
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
        .by_blocks(powers_of_two())
        .filter_map(|o| o)
        .next()
}
