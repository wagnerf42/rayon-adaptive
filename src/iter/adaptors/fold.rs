//! Fold and avoid local reductions.
use crate::prelude::*;
use crate::Policy;
use derive_divisible::Divisible;
use std::option::IntoIter;

/// The `Fold` struct is a parallel folder, returned by the `fold` method on `ParallelIterator`.
/// It is for use when the reduction operation comes with overhead.
/// So instead of reducing all tiny pieces created by local iterators we just
/// reduce for the real divisions.
#[derive(Divisible)]
#[power(I::Power)]
#[trait_bounds(
    I: ParallelIterator,
    O: Send,
    ID: Fn() -> O + Clone + Send,
    F: Fn(O, I::Item) -> O + Clone + Send,
)]
pub struct Fold<I, O, ID, F> {
    pub(crate) remaining_input: I,
    #[divide_by(default)]
    pub(crate) current_output: Option<O>,
    #[divide_by(clone)]
    pub(crate) identity: ID,
    #[divide_by(clone)]
    pub(crate) fold_op: F,
}

impl<
        I: ParallelIterator,
        O: Send,
        ID: Fn() -> O + Clone + Send,
        F: Fn(O, I::Item) -> O + Clone + Send,
    > ParallelIterator for Fold<I, O, ID, F>
{
    type Item = O;
    type SequentialIterator = IntoIter<O>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let final_call = self.base_length().expect("cannot fold infinite sizes") == size;
        let sequential_iterator = self.remaining_input.extract_iter(size);
        let current_output = self.current_output.take().unwrap_or_else(&self.identity);
        let new_output = sequential_iterator.fold(current_output, &self.fold_op);
        if final_call {
            Some(new_output)
        } else {
            self.current_output = Some(new_output); // we put it back here
            None
        }
        .into_iter()
    }

    fn to_sequential(mut self) -> Self::SequentialIterator {
        let sequential_iterator = self.remaining_input.to_sequential();
        let current_output = self.current_output.take().unwrap_or_else(&self.identity);
        let new_output = sequential_iterator.fold(current_output, &self.fold_op);
        Some(new_output).into_iter()
    }

    fn policy(&self) -> Policy {
        self.remaining_input.policy()
    }
    fn blocks_sizes(&mut self) -> Box<dyn Iterator<Item = usize>> {
        self.remaining_input.blocks_sizes()
    }
}
