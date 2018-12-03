use iterator::{AdaptiveIterator, DivisibleIterator};
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};
use std::marker::PhantomData;
use Folder;

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
        let (todo, remaining) = i.split_at(limit);
        (todo.into_iter().fold(io, &self.fold_op), remaining)
    }
    fn to_output(&self, io: Self::IntermediateOutput, _i: Self::Input) -> Self::Output {
        io
    }
}

pub struct IteratorFold<
    I: IndexedParallelIterator + Clone + Sync,
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Send + Sync,
    F: Fn(IO, I::Item) -> IO + Send + Sync,
> {
    pub(crate) identity_op: ID,
    pub(crate) fold_op: F,
    pub(crate) phantom: PhantomData<I>,
}

impl<
        I: IndexedParallelIterator + Clone + Sync,
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Send + Sync,
        F: Fn(IO, I::Item) -> IO + Send + Sync,
    > Folder for IteratorFold<I, IO, ID, F>
{
    type Input = DivisibleIterator<I>;
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
        let mut v: Vec<IO> = i
            .inner_iter
            .clone()
            .skip(i.range.0)
            .take(limit)
            .with_min_len(limit)
            .fold(|| io.clone(), |acc, x| (self.fold_op)(acc, x))
            .collect();
        (
            v.pop().unwrap(),
            DivisibleIterator {
                inner_iter: i.inner_iter,
                range: (i.range.0 + limit, i.range.1),
            },
        )
    }
    fn to_output(&self, io: Self::IntermediateOutput, _i: Self::Input) -> Self::Output {
        io
    }
}
