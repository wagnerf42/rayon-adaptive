//! Fold and avoid local reductions.
use crate::prelude::*;
use crate::Policy;
use std::marker::PhantomData;
use std::option::IntoIter;

/// The `Fold` struct is a parallel folder, returned by the `fold` method on `ParallelIterator`.
/// It is for use when the reduction operation comes with overhead.
/// So instead of reducing all tiny pieces created by local iterators we just
/// reduce for the real divisions.
pub struct Fold<P, I, O, ID, F> {
    pub(crate) remaining_input: I,
    pub(crate) current_output: Option<O>,
    pub(crate) identity: ID,
    pub(crate) fold_op: F,
    pub(crate) phantom: PhantomData<P>,
}

impl<
        P: Power,
        I: ParallelIterator<P>,
        O: Send,
        ID: Fn() -> O + Clone + Send,
        F: Fn(O, I::Item) -> O + Clone + Send,
    > Divisible<P> for Fold<P, I, O, ID, F>
{
    fn base_length(&self) -> Option<usize> {
        self.remaining_input.base_length()
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left, right) = self.remaining_input.divide_at(index);
        self.remaining_input = left;
        let right_folder = Fold {
            remaining_input: right,
            current_output: Some((self.identity)()),
            identity: self.identity.clone(),
            fold_op: self.fold_op.clone(),
            phantom: PhantomData,
        };
        (self, right_folder)
    }
}

impl<
        P: Power,
        I: ParallelIterator<P>,
        O: Send,
        ID: Fn() -> O + Clone + Send,
        F: Fn(O, I::Item) -> O + Clone + Send,
    > Edible for Fold<P, I, O, ID, F>
{
    type Item = O;
    type SequentialIterator = IntoIter<O>;
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let final_call = self.base_length().expect("cannot fold infinite sizes") == size;
        let (sequential_iterator, new_remaining_input) = self.remaining_input.iter(size);
        let current_output = self.current_output.take().unwrap();
        let new_output = sequential_iterator.fold(current_output, &self.fold_op);
        self.remaining_input = new_remaining_input;
        (
            if final_call {
                Some(new_output)
            } else {
                self.current_output = Some(new_output); // we put it back here
                None
            }
            .into_iter(),
            self,
        )
    }
    fn policy(&self) -> Policy {
        self.remaining_input.policy()
    }
}

impl<
        P: Power,
        I: ParallelIterator<P>,
        O: Send,
        ID: Fn() -> O + Clone + Send,
        F: Fn(O, I::Item) -> O + Clone + Send,
    > ParallelIterator<P> for Fold<P, I, O, ID, F>
{
}
