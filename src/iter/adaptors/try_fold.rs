//! Try fold and stop when error
use crate::iter::Try;
use crate::prelude::*;
use crate::schedulers_interruptible::try_fold;
use std::option::IntoIter;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// The `TryFold` struct is similar to the `Fold` struct, except it will
/// stop whenever it encounters an error during the internal fold.
pub struct TryFold<I, U: Try, ID, F> {
    pub(crate) iterator: I,
    pub(crate) current_output: Option<U::Ok>,
    pub(crate) identity: ID,
    pub(crate) fold_op: F,
    pub(crate) is_stopped: Arc<AtomicBool>,
}

impl<I, U, ID, F> Divisible for TryFold<I, U, ID, F>
where
    I: ParallelIterator,
    U: Try + Send,
    F: Fn(U::Ok, I::Item) -> U + Sync + Send + Clone,
    ID: Fn() -> U::Ok + Sync + Send + Clone,
{
    type Power = I::Power;

    fn base_length(&self) -> Option<usize> {
        if self.is_stopped.load(Ordering::Relaxed) {
            Some(0)
        } else {
            self.iterator.base_length()
        }
    }

    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left, right) = self.iterator.divide_at(index);
        self.iterator = left;

        let right = TryFold {
            iterator: right,
            current_output: None,
            identity: self.identity.clone(),
            fold_op: self.fold_op.clone(),
            is_stopped: self.is_stopped.clone(),
        };

        (self, right)
    }
}

impl<I, U, ID, F> ParallelIterator for TryFold<I, U, ID, F>
where
    I: ParallelIterator,
    U: Try + Send,
    F: Fn(U::Ok, I::Item) -> U + Sync + Send + Clone,
    ID: Fn() -> U::Ok + Sync + Send + Clone,
    U::Ok: Send,
{
    type Item = U;
    type SequentialIterator = IntoIter<U>;

    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let mut sequential_iterator = self.iterator.extract_iter(size);
        let current_output = self.current_output.take().unwrap_or_else(&self.identity);

        let new_output = try_fold(&mut sequential_iterator, current_output, &self.fold_op);

        match new_output.into_result() {
            Ok(value) => {
                self.current_output = Some(value);
                None
            }
            Err(error) => {
                self.is_stopped.store(true, Ordering::Relaxed);
                Some(Try::from_error(error))
            }
        }
        .into_iter()
    }

    fn to_sequential(mut self) -> Self::SequentialIterator {
        let current_output = self.current_output.take().unwrap_or_else(&self.identity);

        let mut sequential_iterator = self.iterator.to_sequential();

        let new_output = try_fold(&mut sequential_iterator, current_output, &self.fold_op);

        match new_output.into_result() {
            Ok(output) => Some(Try::from_ok(output)),
            Err(err) => Some(Try::from_error(err)),
        }
        .into_iter()
    }
}
