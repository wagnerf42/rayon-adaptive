use crate::{Divisible, Folder};
use std::marker::PhantomData;

// *WorkFold* is obtained by calling the *work* function on some *Divisible* input.
#[must_use = "folders are lazy and do nothing unless consumed"]
pub struct WorkFold<I: Divisible, WF: Fn(I, usize) -> I + Sync> {
    pub(crate) work_function: WF,
    pub(crate) phantom: PhantomData<I>,
}

impl<I, WF> Folder for WorkFold<I, WF>
where
    I: Divisible,
    WF: Fn(I, usize) -> I + Sync,
{
    type Input = I;
    type IntermediateOutput = ();
    type Output = I;
    fn identity(&self) -> Self::IntermediateOutput {
        ()
    }
    fn fold(
        &self,
        _io: Self::IntermediateOutput,
        i: Self::Input,
        limit: usize,
    ) -> (Self::IntermediateOutput, Self::Input) {
        ((), (self.work_function)(i, limit))
    }
    fn to_output(&self, _io: Self::IntermediateOutput, i: Self::Input) -> Self::Output {
        i
    }
}
