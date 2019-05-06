//! All schedulers are written here.
use crate::prelude::*;

/// reduce parallel iterator
pub(crate) fn schedule<P, I, ID, OP>(iterator: I, identity: &ID, op: &OP) -> I::Item
where
    P: Power,
    I: ParallelIterator<P>,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    schedule_sequential(iterator, identity, op)
}

pub(crate) fn schedule_sequential<P, I, ID, OP>(iterator: I, identity: &ID, op: &OP) -> I::Item
where
    P: Power,
    I: ParallelIterator<P>,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    let full_length = iterator
        .base_length()
        .expect("running on infinite iterator");
    let (_remaining, seq_iter) = iterator.iter(full_length);
    seq_iter.fold(identity(), op)
}

pub(crate) fn schedule_join<P, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
) -> I::Item
where
    P: Power,
    I: ParallelIterator<P>,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    let full_length = iterator
        .base_length()
        .expect("running on infinite iterator");
    if full_length <= sequential_fallback {
        schedule_sequential(iterator, identity, op)
    } else {
        let (left, right) = iterator.divide();
        let (left_result, right_result) = rayon::join(
            || schedule_join(left, identity, op, sequential_fallback),
            || schedule_join(right, identity, op, sequential_fallback),
        );
        op(left_result, right_result)
    }
}
