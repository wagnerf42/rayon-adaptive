use crate::{Divisible, DivisibleIntoBlocks, Folder};
use std::marker::PhantomData;

#[must_use = "folders are lazy and do nothing unless consumed"]
pub struct CuttingFold<I: Divisible, IO: Send, ID: Fn() -> IO, FF: Fn(IO, I) -> IO> {
    pub(crate) identity_op: ID,
    pub(crate) fold_op: FF,
    pub(crate) phantom: PhantomData<I>,
}

impl<I, IO, ID, FF> Folder for CuttingFold<I, IO, ID, FF>
where
    I: DivisibleIntoBlocks + Sync,
    IO: Send + Sync,
    ID: Fn() -> IO + Sync,
    FF: Fn(IO, I) -> IO + Sync,
{
    type Input = I;
    type IntermediateOutput = IO;
    type Output = IO;

    fn identity(&self) -> Self::IntermediateOutput {
        (self.identity_op)()
    }
    fn fold(
        &self,
        io: Self::IntermediateOutput,
        i: Self::Input,
        limit: usize,
    ) -> (Self::IntermediateOutput, Self::Input) {
        let (todo, remaining) = i.divide_at(limit);
        ((self.fold_op)(io, todo), remaining)
    }
    fn to_output(&self, io: Self::IntermediateOutput, _i: Self::Input) -> Self::Output {
        io
    }
}
