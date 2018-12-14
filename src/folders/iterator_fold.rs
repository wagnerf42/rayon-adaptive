use crate::iter::AdaptiveIterator;
use crate::Folder;
use std::marker::PhantomData;

#[must_use = "folders are lazy and do nothing unless consumed"]
pub struct AdaptiveIteratorFold<
    I: AdaptiveIterator,
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Send + Sync,
    F: Fn(IO, I::Item) -> IO + Send + Sync,
> {
    pub(crate) identity_op: ID,
    pub(crate) fold_op: F,
    pub(crate) phantom: PhantomData<I>,
}

impl<
        I: AdaptiveIterator,
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Send + Sync,
        F: Fn(IO, I::Item) -> IO + Send + Sync,
    > Folder for AdaptiveIteratorFold<I, IO, ID, F>
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
        // for now we use the overhead version
        // we could avoid it with a "partial_fold"
        let (todo, remaining) = i.divide_at(limit);
        (todo.into_iter().fold(io, &self.fold_op), remaining)
    }
    fn to_output(&self, io: Self::IntermediateOutput, _i: Self::Input) -> Self::Output {
        io
    }
}
