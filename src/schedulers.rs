//! All schedulers are written here.
use crate::prelude::*;
use crate::small_channel::small_channel;
use crate::utils::power_sizes;
use crate::Policy;
use std::cmp::min;

/// reduce parallel iterator
pub(crate) fn schedule<I, ID, OP, B>(
    scheduling_policy: Policy,
    blocks: &mut B,
    identity: &ID,
    op: &OP,
) -> I::Item
where
    I: ParallelIterator,
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
            Policy::Adaptive(_, _) => schedule_adaptive(b, identity, op, identity()),
            Policy::DefaultPolicy => schedule_rayon(
                b,
                identity,
                op,
                1,
                (rayon::current_num_threads() as f64).log(2.0).ceil() as usize,
            ),
        })
        .fold(identity(), op)
}

fn schedule_sequential<I, ID, OP>(iterator: I, identity: &ID, op: &OP) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    iterator.to_sequential().fold(identity(), op)
}

fn schedule_join<I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
) -> I::Item
where
    I: ParallelIterator,
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

pub(crate) fn schedule_rayon<I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
    counter: usize,
) -> I::Item
where
    I: ParallelIterator,
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
        if right.base_length().unwrap_or(1) == 0 {
            // basic iterators don't know their sizes
            // we need to divide and check if the division failed
            schedule_sequential(left, identity, op)
        } else {
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
}

pub(crate) fn schedule_adaptive<I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    output: I::Item,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    let (sender, receiver) = small_channel();
    let (min_size, max_size) = if let Policy::Adaptive(min_size, max_size) = iterator.policy() {
        (min_size, max_size)
    } else {
        unreachable!()
    };
    let (left_result, maybe_right_result): (I::Item, Option<I::Item>) = rayon::join_context(
        |_| match power_sizes(min_size, max_size)
            .take_while(|_| !sender.receiver_is_waiting())
            .try_fold((iterator, output), |(mut iterator, output), s| {
                let checked_size = min(s, iterator.base_length().expect("infinite iterator"));
                let sequential_iterator = iterator.extract_iter(checked_size);
                let new_output;
                #[cfg(feature = "logs")]
                {
                    new_output = rayon_logs::subgraph("adaptive block", checked_size, || {
                        sequential_iterator.fold(output, op)
                    })
                }
                #[cfg(not(feature = "logs"))]
                {
                    new_output = sequential_iterator.fold(output, op)
                }
                if iterator
                    .base_length()
                    .expect("running on infinite iterator")
                    == 0
                {
                    Err((iterator, new_output))
                } else {
                    Ok((iterator, new_output))
                }
            }) {
            Ok((remaining_iterator, output)) => {
                let full_length = remaining_iterator
                    .base_length()
                    .expect("running on infinite iterator");
                if full_length <= min_size {
                    sender.send(None);
                    remaining_iterator.to_sequential().fold(output, op)
                } else {
                    let (my_half, his_half) = remaining_iterator.divide();
                    sender.send(Some(his_half));
                    schedule_adaptive(my_half, identity, op, output)
                }
            }
            Err((_, output)) => {
                sender.send(None);
                output
            }
        },
        |c| {
            if c.migrated() {
                receiver
                    .recv()
                    .expect("receiving adaptive iterator failed")
                    .map(|iterator| schedule_adaptive(iterator, identity, op, identity()))
            } else {
                None
            }
        },
    );
    if let Some(right_result) = maybe_right_result {
        op(left_result, right_result)
    } else {
        left_result
    }
}
