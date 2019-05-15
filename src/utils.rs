//! misc utilities.
use std::iter::{repeat, successors};

/// Iterator on min_size, min_size*2, min_size*4, ..., max_size, max_size, max_size...
pub(crate) fn power_sizes(min_size: usize, max_size: usize) -> impl Iterator<Item = usize> {
    successors(Some(min_size), |&p| Some(2 * p))
        .take_while(move |&p| p < max_size)
        .chain(repeat(max_size))
}
