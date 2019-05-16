//! Work locally.
use crate::prelude::*;
use derive_divisible::Divisible;
use std::marker::PhantomData;
use std::option::IntoIter;

/// The `Work` struct is returned by the `work` method on any `Divisible`.
/// It slowly consumes the input piece by piece.
#[derive(Divisible)]
#[power(P)]
pub struct Work<P: Power, I: Divisible<P>, W: Clone> {
    pub(crate) remaining_input: Option<I>,
    #[divide_by(clone)]
    pub(crate) work_op: W,
    #[divide_by(default)]
    pub(crate) phantom: PhantomData<P>,
}

impl<P: Power, I: Divisible<P> + Send, W: Fn(I, usize) -> I + Send + Clone> Edible
    for Work<P, I, W>
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

impl<P: Power, I: Divisible<P> + Send, W: Fn(I, usize) -> I + Send + Clone> ParallelIterator<P>
    for Work<P, I, W>
{
}
