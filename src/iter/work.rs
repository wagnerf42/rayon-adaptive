//! Work locally.
use crate::prelude::*;
use std::marker::PhantomData;
use std::option::IntoIter;

/// The `Work` struct is returned by the `work` method on any `Divisible`.
/// It slowly consumes the input piece by piece.
pub struct Work<P: Power, I: Divisible<P>, W: Clone> {
    pub(crate) remaining_input: Option<I>,
    pub(crate) work_op: W,
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: Divisible<P>, W: Fn(I, usize) -> I + Send + Clone> Divisible<P::NotIndexed>
    for Work<P, I, W>
{
    fn base_length(&self) -> Option<usize> {
        if self.remaining_input.is_none() {
            Some(0)
        } else {
            self.remaining_input.as_ref().unwrap().base_length()
        }
    }
    fn divide_at(mut self, index: usize) -> (Self, Self) {
        let (left, right) = self.remaining_input.unwrap().divide_at(index);
        self.remaining_input = Some(left);
        let right_work = Work {
            remaining_input: Some(right),
            work_op: self.work_op.clone(),
            phantom: PhantomData,
        };
        (self, right_work)
    }
}

impl<P: Power, I: Divisible<P> + Send, W: Fn(I, usize) -> I + Send + Clone>
    ParallelIterator<P::NotIndexed> for Work<P, I, W>
{
    type Item = I;
    type SequentialIterator = IntoIter<I>;
    fn iter(mut self, size: usize) -> (Self::SequentialIterator, Self) {
        let final_call = self.base_length().expect("cannot fold infinite sizes") == size;
        let new_input = (self.work_op)(self.remaining_input.take().unwrap(), size);
        (
            if final_call {
                Some(new_input)
            } else {
                self.remaining_input = Some(new_input);
                None
            }
            .into_iter(),
            self,
        )
    }
}
