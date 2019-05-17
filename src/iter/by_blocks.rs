//! `ByBlocks` structure for `ParallelIterator::by_blocks`.
use crate::prelude::*;
use derive_divisible::{Divisible, IntoIterator, ParallelIterator};
use std::marker::PhantomData;

/// Iterator which configured to run on macro blocks. See `ParallelIterator::by_blocks`.
#[derive(Divisible, ParallelIterator, IntoIterator)]
#[power(P)]
#[item(I::Item)]
#[sequential_iterator(I::SequentialIterator)]
#[iterator_extraction(i)]
pub struct ByBlocks<P: Power, I: ParallelIterator<P>> {
    #[divide_by(default)]
    pub(crate) sizes_iterator: Option<Box<Iterator<Item = usize> + Send>>,
    pub(crate) iterator: I,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}
