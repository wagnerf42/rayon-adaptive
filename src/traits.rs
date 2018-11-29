//! This module contains all traits enabling us to express some parallelism.
use rayon::prelude::*;
use scheduling::{schedule, Policy};
use std;
use std::cmp::{max, min};
use std::collections::LinkedList;
use std::marker::PhantomData;
use std::ptr;

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

pub struct Map<F: Folder, O: Send, M: Fn(F::Output) -> O + Sync> {
    inner_folder: F,
    map_op: M,
    phantom: PhantomData<O>,
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

pub struct IteratorFold<
    I: IndexedParallelIterator + Clone + Sync,
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Send + Sync,
    F: Fn(IO, I::Item) -> IO + Send + Sync,
> {
    //input: DivisibleIterator<I>,
    identity_op: ID,
    fold_op: F,
    phantom: PhantomData<I>,
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

pub struct WorkFold<I: Divisible, WF: Fn(I, usize) -> I + Sync> {
    work_function: WF,
    phantom: PhantomData<I>,
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

pub struct Fold<I: Divisible, IO: Send, ID: Fn() -> IO, FF: Fn(IO, I, usize) -> (IO, I)> {
    identity_op: ID,
    fold_op: FF,
    phantom: PhantomData<I>,
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

pub struct ActivatedInput<F: Folder> {
    input: F::Input,
    folder: F,
    initial_block_size: Option<usize>,
}

impl<F: Folder> IntoIterator for ActivatedInput<F> {
    type Item = F::Output;
    type IntoIter = std::collections::linked_list::IntoIter<F::Output>;
    fn into_iter(self) -> Self::IntoIter {
        let (input, folder) = (self.input, self.folder);
        let list_folder = folder.map(|o| {
            let mut l = LinkedList::new();
            l.push_back(o);
            l
        });

        let sequential_limit = (input.len() as f64).log(2.0).ceil() as usize;

        let outputs_list = schedule(
            input,
            &list_folder,
            &|mut left, mut right| {
                left.append(&mut right);
                left
            },
            Policy::Adaptive(sequential_limit),
        );
        outputs_list.into_iter()
    }
}

impl<F: Folder> ActivatedInput<F> {
    /// Sets the initial block size for the adaptive algorithm.
    pub fn initial_block_size(self, block_size: usize) -> Self {
        ActivatedInput {
            input: self.input,
            folder: self.folder,
            initial_block_size: Some(block_size),
        }
    }
    pub fn map<O: Send + Sync, M: Fn(F::Output) -> O + Sync>(
        self,
        map_op: M,
    ) -> ActivatedInput<Map<F, O, M>> {
        ActivatedInput {
            input: self.input,
            folder: self.folder.map(map_op),
            initial_block_size: self.initial_block_size,
        }
    }
    pub fn reduce<RF: Fn(F::Output, F::Output) -> F::Output + Sync>(
        self,
        reduce_function: RF,
        policy: Policy,
    ) -> F::Output {
        let (input, folder) = (self.input, self.folder);
        schedule(input, &folder, &reduce_function, policy)
    }
}

// TODO: why on earth do I need Sync on I ?
impl<I: DivisibleAtIndex, F: Folder<Input = I>> ActivatedInput<F> {
    pub fn by_blocks<S: Iterator<Item = usize>>(
        self,
        blocks_sizes: S,
    ) -> impl Iterator<Item = F::Output> {
        let (input, folder) = (self.input, self.folder);

        let list_folder = folder.map(|o| {
            let mut l = LinkedList::new();
            l.push_back(o);
            l
        });

        input.chunks(blocks_sizes).flat_map(move |input| {
            let sequential_limit = (input.len() as f64).log(2.0).ceil() as usize;

            let outputs_list = schedule(
                input,
                &list_folder,
                &|mut left, mut right| {
                    left.append(&mut right);
                    left
                },
                Policy::Adaptive(sequential_limit),
            );
            outputs_list.into_iter()
        })
    }
}

pub trait AdaptiveFolder: IndexedParallelIterator {
    fn adaptive_fold<IO, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<IteratorFold<Self, IO, ID, F>>
    where
        Self: Sync + IndexedParallelIterator + Clone,
        IO: Send + Sync + Clone,
        ID: Fn() -> IO + Sync + Send + Clone,
        F: Fn(IO, Self::Item) -> IO + Sync + Send + Clone,
    {
        inner_adaptive_fold(self, identity, fold_op)
    }
}

fn inner_adaptive_fold<I, IO, ID, F>(
    iterator: I,
    identity: ID,
    fold_op: F,
) -> ActivatedInput<IteratorFold<I, IO, ID, F>>
where
    I: Sync + IndexedParallelIterator + Clone,
    IO: Send + Sync + Clone,
    ID: Fn() -> IO + Sync + Send + Clone,
    F: Fn(IO, I::Item) -> IO + Sync + Send + Clone,
{
    let range = (0, iterator.len());
    let divisible_input = DivisibleIterator {
        inner_iter: iterator,
        range,
    };
    let iter_fold = IteratorFold {
        identity_op: identity,
        fold_op,
        phantom: PhantomData,
    };
    ActivatedInput {
        input: divisible_input,
        folder: iter_fold,
        initial_block_size: None,
    }
}

impl<I> AdaptiveFolder for I where I: IndexedParallelIterator + Sync + Clone {}

pub trait Divisible: Sized + Send + Sync {
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Return our length.
    fn len(&self) -> usize;
    /// Is there something left to do ?
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn work<WF: Fn(Self, usize) -> Self + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<WorkFold<Self, WF>> {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self,
            folder,
            initial_block_size: None,
        }
    }
    fn fold<O, ID, F>(self, identity: ID, fold_op: F) -> ActivatedInput<Fold<Self, O, ID, F>>
    where
        O: Send + Sync,
        ID: Fn() -> O + Sync,
        F: Fn(O, Self, usize) -> (O, Self) + Sync,
    {
        let folder = Fold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self,
            folder,
            initial_block_size: None,
        }
    }
    /// Easy api when we return no results.
    fn for_each<WF>(self, work_function: WF, policy: Policy)
    where
        WF: Fn(Self, usize) -> Self + Sync,
    {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }.map(|_| ());
        let reduce = |_, _| ();
        schedule(self, &folder, &reduce, policy)
    }
}

pub struct Chunks<I: DivisibleAtIndex, S: Iterator<Item = usize>> {
    remaining: I,
    remaining_sizes: S,
}

pub struct DivisibleIterator<I>
where
    I: IndexedParallelIterator + Clone + Sync,
{
    inner_iter: I,
    range: (usize, usize),
}

impl<I> Divisible for DivisibleIterator<I>
where
    I: IndexedParallelIterator + Clone + Sync,
{
    fn split(self) -> (Self, Self) {
        let left_iter = self.inner_iter.clone();
        let right_iter = self.inner_iter;
        (
            DivisibleIterator {
                inner_iter: left_iter,
                range: (self.range.0, (self.range.0 + self.range.1) / 2 as usize),
            },
            DivisibleIterator {
                inner_iter: right_iter,
                range: (
                    (self.range.1 + self.range.0) / 2 as usize + 1,
                    self.range.1 as usize,
                ),
            },
        )
    }

    fn len(&self) -> usize {
        if self.range.1 > self.range.0 {
            self.range.1 - self.range.0
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        self.range.1 == self.range.0
    }
}

impl<I: DivisibleAtIndex, S: Iterator<Item = usize>> Iterator for Chunks<I, S> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.len() == 0 {
            None
        } else {
            let next_size = min(
                self.remaining_sizes
                    .next()
                    .expect("not enough sizes for chunks"),
                self.remaining.len(),
            );
            let next_chunk = self.remaining.cut_left_at(next_size);
            Some(next_chunk)
        }
    }
}

pub trait DivisibleAtIndex: Divisible {
    /// Divide ourselves where requested.
    fn split_at(self, index: usize) -> (Self, Self);
    /// Divide ourselves keeping right part in self.
    /// Returns the left part.
    /// NB: this is useful for iterators creation.
    fn cut_left_at(&mut self, index: usize) -> Self {
        // there is a lot of unsafe going on here.
        // I think it's ok. rust uses the same trick for moving iterators (vecs for example)
        unsafe {
            let my_copy = ptr::read(self);
            let (left, right) = my_copy.split_at(index);
            let pointer_to_self = self as *mut Self;
            ptr::write(pointer_to_self, right);
            left
        }
    }
    /// Get a sequential iterator on chunks of Self of given sizes.
    fn chunks<S: Iterator<Item = usize>>(self, sizes: S) -> Chunks<Self, S> {
        Chunks {
            remaining: self,
            remaining_sizes: sizes,
        }
    }
    /// Easy api but use only when splitting generates no tangible work overhead.
    fn map_reduce<MF, RF, O>(
        self,
        map_function: MF,
        reduce_function: RF,
        initial_block_size: usize,
    ) -> O
    where
        MF: Fn(Self) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send + Sync,
    {
        let folder = Fold {
            identity_op: || None,
            fold_op: |o: Option<O>, i: Self, limit: usize| -> (Option<O>, Self) {
                let (todo_now, remaining) = i.split_at(limit);
                let new_result = map_function(todo_now);
                (
                    if let Some(output) = o {
                        Some(reduce_function(output, new_result))
                    } else {
                        Some(new_result)
                    },
                    remaining,
                )
            },
            phantom: PhantomData,
        }.map(|o| o.unwrap());

        schedule(
            self,
            &folder,
            &|left, right| reduce_function(left, right),
            Policy::Adaptive(initial_block_size),
        )
    }
}

impl<'a, T: Sync> Divisible for &'a [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at(mid)
    }
}

impl<'a, T: Sync> DivisibleAtIndex for &'a [T] {
    fn split_at(self, index: usize) -> (Self, Self) {
        self.split_at(index)
    }
}

//TODO: I don't get why the compiler requires send here
impl<'a, T: 'a + Sync + Send> Divisible for &'a mut [T] {
    fn len(&self) -> usize {
        (*self as &[T]).len()
    }
    fn split(self) -> (Self, Self) {
        let mid = self.len() / 2;
        self.split_at_mut(mid)
    }
}

impl<'a, T: 'a + Sync + Send> DivisibleAtIndex for &'a mut [T] {
    fn split_at(self, index: usize) -> (Self, Self) {
        self.split_at_mut(index)
    }
}

//TODO: macroize all that stuff ; even better : derive ?
impl<A: Divisible, B: Divisible> Divisible for (A, B) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), self.1.len())
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        ((left_a, left_b), (right_a, right_b))
    }
}

//TODO: macroize all that stuff ; even better : derive ?
impl<A: DivisibleAtIndex, B: DivisibleAtIndex> DivisibleAtIndex for (A, B) {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left_a, right_a) = self.0.split_at(index);
        let (left_b, right_b) = self.1.split_at(index);
        ((left_a, left_b), (right_a, right_b))
    }
}

impl<A: Divisible, B: Divisible, C: Divisible> Divisible for (A, B, C) {
    fn len(&self) -> usize {
        std::cmp::min(self.0.len(), std::cmp::min(self.1.len(), self.2.len()))
    }
    fn split(self) -> (Self, Self) {
        let (left_a, right_a) = self.0.split();
        let (left_b, right_b) = self.1.split();
        let (left_c, right_c) = self.2.split();
        ((left_a, left_b, left_c), (right_a, right_b, right_c))
    }
}
