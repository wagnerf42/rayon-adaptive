//! Folder trait and all its implementations.
use crate::Divisible;
use std::marker::PhantomData;
mod map;
pub use self::map::Map;
pub(crate) mod cutting_fold;
pub(crate) mod fold;
pub(crate) mod iterator_fold;
pub(crate) mod work_fold;

// The *Folder* trait enables us to abstract other adaptive operations.
// it takes a *Divisible* input, recursively cuts into smaller inputs,
// fold producing intermediate outputs, maps these to final outputs.
// These outputs are then ready for a final reduction.
// Note that all that stuff is lazy and nothing takes place before the reduction.
pub trait Folder: Sync + Sized {
    type Input: Divisible;
    type IntermediateOutput: Send + Sync;
    type Output: Send + Sync;
    fn identity(&self) -> Self::IntermediateOutput;
    fn fold(
        &self,
        io: Self::IntermediateOutput,
        i: Self::Input,
        limit: usize,
    ) -> (Self::IntermediateOutput, Self::Input);
    fn to_output(&self, io: Self::IntermediateOutput, i: Self::Input) -> Self::Output;
    fn map<O: Send, M: Fn(Self::Output) -> O + Sync>(self, map_op: M) -> Map<Self, O, M> {
        Map {
            inner_folder: self,
            map_op,
            phantom: PhantomData,
        }
    }
}
