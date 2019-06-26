//! All schedulers are written here.
use crate::iter::Try;
use crate::prelude::*;
use crate::small_channel::small_channel;
use crate::utils::power_sizes;
use crate::Policy;
use std::cmp::min;
use std::iter::successors;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering; // nightly
/// reduce parallel iterator

pub(crate) fn schedule_interruptible<T, I, ID, OP>(
    scheduling_policy: Policy,
    mut par_iter: I,
    identity: &ID,
    op: &OP,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(T, T) -> I::Item + Sync,
    ID: Fn() -> T + Sync,
    I::Item: Try<Ok = T>,
{
    let not_failed = AtomicBool::new(true);
    let sizes_block = par_iter.blocks_sizes();
    let sizes = sizes_block.chain(successors(Some(10_000usize * 2), |n| n.checked_mul(2)));
    try_fold(
        &mut par_iter.blocks(sizes).map(|b| match scheduling_policy {
            Policy::Join(sequential_fallback) => {
                schedule_join(b, identity, op, sequential_fallback, &not_failed)
            }
            Policy::Rayon(sequential_fallback) => schedule_rayon(
                b,
                identity,
                op,
                sequential_fallback,
                (rayon::current_num_threads() as f64).log(2.0).ceil() as usize + 2,
                &not_failed,
            ),
            Policy::Sequential => schedule_sequential(b, identity, op, &not_failed),
            Policy::Adaptive(_, _) => schedule_adaptive_interruptible(
                b,
                identity,
                op,
                I::Item::from_ok(identity()),
                &not_failed,
            ),
            Policy::DefaultPolicy => {
                let size_entry = b.base_length().expect("infinite iterator"); // TODO Verify the policy should be for each blocks or not
                let p = rayon::current_num_threads();
                let policy = Policy::Adaptive(((size_entry as f32).log2() * 2.0) as usize, 10_000);
                if (((p as f64).log(2.0).ceil() as usize) * p * p * 100) < size_entry {
                    schedule_adaptive_interruptible(
                        b.with_policy(policy),
                        identity,
                        op,
                        I::Item::from_ok(identity()),
                        &not_failed,
                    )
                } else {
                    schedule_rayon(
                        b,
                        identity,
                        op,
                        1,
                        (rayon::current_num_threads() as f64).log(2.0).ceil() as usize + 2,
                        &not_failed,
                    )
                }
            }
        }),
        identity(),
        |b, i| match i.into_result() {
            Ok(t) => op(b, t),
            Err(e) => I::Item::from_error(e),
        },
    )
}

fn schedule_sequential<T, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    not_failed: &AtomicBool,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(T, T) -> I::Item + Sync,
    ID: Fn() -> T + Sync,
    I::Item: Try<Ok = T>,
{
    try_fold(&mut iterator.to_sequential(), identity(), |b, i| {
        match i.into_result() {
            Ok(t) => op(b, t),
            Err(e) => {
                not_failed.store(false, Ordering::Relaxed);
                I::Item::from_error(e)
            }
        }
    })
}

fn schedule_join<T, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
    not_failed: &AtomicBool,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(T, T) -> I::Item + Sync,
    ID: Fn() -> T + Sync,
    I::Item: Try<Ok = T>,
{
    let full_length = iterator
        .base_length()
        .expect("running on infinite iterator");
    if full_length <= sequential_fallback {
        schedule_sequential(iterator, identity, op, &not_failed)
    } else {
        let (left, right) = iterator.divide();
        let (left_value, possible_right_value) = rayon::join(
            || schedule_join(left, identity, op, sequential_fallback, &not_failed),
            || {
                if not_failed.load(Ordering::Relaxed) {
                    Some(schedule_join(
                        right,
                        identity,
                        op,
                        sequential_fallback,
                        &not_failed,
                    ))
                } else {
                    None
                }
            },
        );

        match left_value.into_result() {
            Ok(tleft) => {
                if let Some(right_value) = possible_right_value {
                    match right_value.into_result() {
                        Ok(tright) => {
                            if not_failed.load(Ordering::Relaxed) {
                                op(tleft, tright)
                            } else {
                                I::Item::from_ok(tleft)
                            }
                        }
                        Err(e) => I::Item::from_error(e),
                    }
                } else {
                    I::Item::from_ok(tleft)
                }
            }
            Err(el) => I::Item::from_error(el),
        }
    }
}

fn schedule_rayon<T, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    sequential_fallback: usize,
    counter: usize,
    not_failed: &AtomicBool,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(T, T) -> I::Item + Sync,
    ID: Fn() -> T + Sync,
    I::Item: Try<Ok = T>,
{
    let full_length = iterator
        .base_length()
        .expect("running on infinite iterator");
    if full_length <= sequential_fallback || counter == 0 {
        schedule_sequential(iterator, identity, op, &not_failed)
    } else {
        let (left, right) = iterator.divide();
        let (left_value, possible_right_value) = rayon::join_context(
            |_| {
                schedule_rayon(
                    left,
                    identity,
                    op,
                    sequential_fallback,
                    counter - 1,
                    &not_failed,
                )
            },
            |c| {
                if not_failed.load(Ordering::Relaxed) {
                    Some(schedule_rayon(
                        right,
                        identity,
                        op,
                        sequential_fallback,
                        if c.migrated() {
                            (rayon::current_num_threads() as f64).log(2.0).ceil() as usize + 2
                        } else {
                            counter - 1
                        },
                        &not_failed,
                    ))
                } else {
                    None
                }
            },
        );
        match left_value.into_result() {
            Ok(tleft) => {
                if let Some(right_value) = possible_right_value {
                    match right_value.into_result() {
                        Ok(tright) => {
                            if not_failed.load(Ordering::Relaxed) {
                                op(tleft, tright)
                            } else {
                                I::Item::from_ok(tleft)
                            }
                        }
                        Err(e) => I::Item::from_error(e),
                    }
                } else {
                    I::Item::from_ok(tleft)
                }
            }
            Err(el) => I::Item::from_error(el),
        }
    }
}

pub(crate) fn schedule_adaptive_interruptible<T, I, ID, OP>(
    iterator: I,
    identity: &ID,
    op: &OP,
    output: I::Item,
    not_failed: &AtomicBool,
) -> I::Item
where
    I: ParallelIterator,
    OP: Fn(T, T) -> I::Item + Sync,
    ID: Fn() -> T + Sync,
    I::Item: Try<Ok = T>,
{
    let (sender, receiver) = small_channel();
    let (min_size, max_size) = if let Policy::Adaptive(min_size, max_size) = iterator.policy() {
        (min_size, max_size)
    } else {
        unreachable!()
    };
    let (left_answer, maybe_right_answer): (I::Item, Option<I::Item>) = rayon::join_context(
        |_| match power_sizes(min_size, max_size)
            .take_while(|_| !sender.receiver_is_waiting() && (*not_failed).load(Ordering::Relaxed))
            .try_fold((iterator, output), |(mut iterator, output), s| {
                let checked_size = min(s, iterator.base_length().expect("infinite iterator"));
                let mut sequential_iterator = iterator.extract_iter(checked_size);
                let new_output = if not_failed.load(Ordering::Relaxed) {
                    {
                        match output.into_result() {
                            Ok(e) => try_fold(&mut sequential_iterator, e, |b, i| {
                                match i.into_result() {
                                    Ok(t) => op(b, t),
                                    Err(e) => {
                                        not_failed.store(false, Ordering::Relaxed);
                                        I::Item::from_error(e)
                                    }
                                }
                            }),
                            Err(e) => I::Item::from_error(e),
                        }
                    }
                } else {
                    output
                };
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
                if full_length <= min_size && (*not_failed).load(Ordering::Relaxed) {
                    sender.send(None);

                    match output.into_result() {
                        Ok(e) => {
                            try_fold(&mut remaining_iterator.to_sequential(), e, |b, i| {
                                match i.into_result() {
                                    Ok(t) => op(b, t),
                                    Err(e) => I::Item::from_error(e),
                                }
                            })
                        }
                        Err(e) => I::Item::from_error(e),
                    }
                } else if (*not_failed).load(Ordering::Relaxed) {
                    let (my_half, his_half) = remaining_iterator.divide();
                    sender.send(Some(his_half));
                    schedule_adaptive_interruptible(my_half, identity, op, output, &not_failed)
                } else {
                    sender.send(None);
                    output
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
                    .map(|iterator| {
                        schedule_adaptive_interruptible(
                            iterator,
                            identity,
                            op,
                            I::Item::from_ok(identity()),
                            not_failed,
                        )
                    })
            } else {
                None
            }
        },
    );

    if let Some(right_answer) = maybe_right_answer {
        let left_result = left_answer.into_result();
        match left_result {
            Ok(left_value) => {
                let right_result = right_answer.into_result();
                match right_result {
                    Ok(right_value) => {
                        if not_failed.load(Ordering::Relaxed) {
                            op(left_value, right_value)
                        } else {
                            I::Item::from_ok(left_value)
                        }
                    }
                    Err(right_error) => I::Item::from_error(right_error),
                }
            }
            Err(left_error) => I::Item::from_error(left_error),
        }
    } else {
        left_answer
    }
}

pub(crate) fn try_fold<I, B, F, R>(iterator: &mut I, init: B, mut f: F) -> R
where
    F: FnMut(B, I::Item) -> R,
    R: Try<Ok = B>,
    I: Iterator,
{
    let mut accum = init;
    while let Some(x) = iterator.next() {
        let accum_value = f(accum, x);
        match accum_value.into_result() {
            Ok(e) => {
                accum = e;
            }
            Err(e) => return Try::from_error(e),
        }
    }
    Try::from_ok(accum)
}

#[test]
fn test_all_adaptative() {
    assert!(!(1u64..10_000).into_par_iter().all(|x| x != 8_500));
    assert!((1u64..10_000).into_par_iter().all(|x| x > 0));
    assert!(!(1u64..50_000).into_par_iter().all(|x| x != 47_000));
}

#[test]
fn test_all_rayon() {
    assert!(!(0u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Rayon(1))
        .all(|x| x != 8_500));
    assert!(!(1u64..10_000).into_par_iter().all(|x| x != 8_500));
    assert!((1u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Rayon(1))
        .all(|x| x > 0));
    assert!(!(1u64..50_000)
        .into_par_iter()
        .with_policy(Policy::Rayon(1))
        .all(|x| x != 47_000));
}

#[test]
fn test_all_join() {
    assert!(!(1u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Join(2_000))
        .all(|x| x != 8_500));
    assert!((1u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Join(2_000))
        .all(|x| x > 0));
    assert!(!(1u64..50_000)
        .into_par_iter()
        .with_policy(Policy::Join(2_000))
        .all(|x| x != 47_000));
}

#[test]
fn test_all_seq() {
    assert!(!(0u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Sequential)
        .all(|x| x != 8_500));
    assert!((1u64..10_000)
        .into_par_iter()
        .with_policy(Policy::Sequential)
        .all(|x| x > 0));
    assert!(!(1u64..50_000)
        .into_par_iter()
        .with_policy(Policy::Sequential)
        .all(|x| x != 47_000));
}

#[test]
fn test_try_reduce_adaptative() {
    let sums = (0u64..15_000)
        .into_par_iter()
        .map(|e| Some(e))
        .try_reduce(|| 0, |a, b| a.checked_add(b))
        .unwrap();
    assert_eq!(sums, (14_999 * 15_000 / 2));
}
#[test]
fn test_try_reduce_join() {
    let sums = (0u64..15_000)
        .into_par_iter()
        .with_policy(Policy::Join(2_000))
        .map(|e| Some(e))
        .try_reduce(|| 0, |a, b| a.checked_add(b))
        .unwrap();
    assert_eq!(sums, (14_999 * 15_000 / 2));
}
#[test]
fn test_try_reduce_rayon() {
    let sums = (0u64..15_000)
        .into_par_iter()
        .with_policy(Policy::Rayon(1))
        .map(|e| Some(e))
        .try_reduce(|| 0, |a, b| a.checked_add(b))
        .unwrap();
    assert_eq!(sums, (14_999 * 15_000 / 2));
}
