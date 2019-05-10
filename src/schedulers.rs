//! All schedulers are written here.
use crate::prelude::*;
use crate::Policy;

/// reduce parallel iterator
pub(crate) fn schedule<P, I, ID, OP, B>(
    scheduling_policy: Policy,
    blocks: &mut B,
    identity: &ID,
    op: &OP,
) -> I::Item
where
    P: Power,
    I: ParallelIterator<P>,
    B: Iterator<Item = I>,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    blocks
        .map(|b| match scheduling_policy {
            Policy::Join(blocks_sizes) => schedule_join(b, identity, op, blocks_sizes),
            Policy::Rayon => schedule_rayon(b, identity, op),
            Policy::Sequential => schedule_sequential(b, identity, op),
        })
        .fold(identity(), op)
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
    let (seq_iter, _remaining) = iterator.iter(full_length);
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

pub(crate) fn schedule_rayon<P, I, ID, OP>(iterator: I, identity: &ID, op: &OP) -> I::Item
where
    P: Power,
    I: ParallelIterator<P>,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    unimplemented!()
}
