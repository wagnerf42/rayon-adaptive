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
            Policy::Join(sequential_fallback) => {
                schedule_join(b, identity, op, sequential_fallback)
            }
            Policy::Rayon(sequential_fallback) => schedule_rayon(
                b,
                identity,
                op,
                sequential_fallback,
                (rayon::current_num_threads() as f64).log(2.0).ceil() as usize,
            ),
            Policy::Sequential => schedule_sequential(b, identity, op),
        })
        .fold(identity(), op)
}

fn schedule_sequential<P, I, ID, OP>(iterator: I, identity: &ID, op: &OP) -> I::Item
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

fn schedule_join<P, I, ID, OP>(
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

pub(crate) fn schedule_rayon<P, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
    counter: usize,
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
    if full_length <= sequential_fallback || counter == 0 {
        schedule_sequential(iterator, identity, op)
    } else {
        let (left, right) = iterator.divide();
        let (left_result, right_result) = rayon::join_context(
            |_| schedule_rayon(left, identity, op, sequential_fallback, counter - 1),
            |c| {
                schedule_rayon(
                    right,
                    identity,
                    op,
                    sequential_fallback,
                    if c.migrated() {
                        (rayon::current_num_threads() as f64).log(2.0).ceil() as usize + 1 // the +1 mimics rayon's current behaviour
                    } else {
                        counter - 1
                    },
                )
            },
        );
        op(left_result, right_result)
    }
}
