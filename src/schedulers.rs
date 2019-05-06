//! All schedulers are written here.
use crate::iter::BaseIterator;
use crate::prelude::*;

/// reduce parallel iterator
pub(crate) fn schedule<ParIter, ID, OP>(iterator: ParIter, identity: &ID, op: &OP) -> ParIter::Item
where
    ParIter: BaseIterator,
    OP: Fn(ParIter::Item, ParIter::Item) -> ParIter::Item + Sync,
    ID: Fn() -> ParIter::Item + Sync,
{
    schedule_sequential(iterator, identity, op)
}

pub(crate) fn schedule_sequential<ParIter, ID, OP>(
    iterator: ParIter,
    identity: &ID,
    op: &OP,
) -> ParIter::Item
where
    ParIter: BaseIterator,
    OP: Fn(ParIter::Item, ParIter::Item) -> ParIter::Item + Sync,
    ID: Fn() -> ParIter::Item + Sync,
{
    let full_length = iterator.base_length();
    let (_remaining, seq_iter) = iterator.iter(full_length);
    seq_iter.fold(identity(), op)
}

pub(crate) fn schedule_join<ParIter, ID, OP>(
    iterator: ParIter,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
) -> ParIter::Item
where
    ParIter: BaseIterator,
    OP: Fn(ParIter::Item, ParIter::Item) -> ParIter::Item + Sync,
    ID: Fn() -> ParIter::Item + Sync,
{
    let full_length = iterator.base_length();
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
