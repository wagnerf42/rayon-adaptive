use std::marker::PhantomData;
use {Divisible, Folder};

pub struct Fold<I: Divisible, IO: Send, ID: Fn() -> IO, FF: Fn(IO, I, usize) -> (IO, I)> {
    pub(crate) identity_op: ID,
    pub(crate) fold_op: FF,
    pub(crate) phantom: PhantomData<I>,
}

impl<I, IO, ID, FF> Folder for Fold<I, IO, ID, FF>
where
    I: Divisible + Sync,
    IO: Send + Sync,
    ID: Fn() -> IO + Sync,
    FF: Fn(IO, I, usize) -> (IO, I) + Sync,
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
        (self.fold_op)(io, i, limit)
    }
    fn to_output(&self, io: Self::IntermediateOutput, _i: Self::Input) -> Self::Output {
        io
    }
}
