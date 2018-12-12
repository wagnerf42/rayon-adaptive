//! map each folder's output to something else.
use std::marker::PhantomData;
use crate::Folder;

pub struct Map<F: Folder, O: Send, M: Fn(F::Output) -> O + Sync> {
    pub(crate) inner_folder: F,
    pub(crate) map_op: M,
    pub(crate) phantom: PhantomData<O>,
}

impl<F, O, M> Folder for Map<F, O, M>
where
    F: Folder,
    O: Send + Sync,
    M: Fn(F::Output) -> O + Sync,
{
    type Input = F::Input;
    type IntermediateOutput = F::IntermediateOutput;
    type Output = O;
    fn identity(&self) -> Self::IntermediateOutput {
        self.inner_folder.identity()
    }
    fn fold(
        &self,
        io: Self::IntermediateOutput,
        i: Self::Input,
        limit: usize,
    ) -> (Self::IntermediateOutput, Self::Input) {
        self.inner_folder.fold(io, i, limit)
    }
    fn to_output(&self, io: Self::IntermediateOutput, i: Self::Input) -> Self::Output {
        (self.map_op)(self.inner_folder.to_output(io, i))
    }
}
