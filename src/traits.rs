//! This module contains all traits enabling us to express some parallelism.
use scheduling::{schedule, Policy};
use std;
use std::cmp::min;
use std::collections::LinkedList;
use std::iter::repeat;
use std::marker::PhantomData;
use std::mem;
use std::ptr;

pub struct ActivatedInput<
    I: Divisible,
    O: Send,
    WF: Fn(I, usize) -> I + Sync,
    MF: Fn(I) -> O + Sync,
> {
    input: I,
    work_function: WF,
    map_function: MF, // TODO: rename to map
    output_type: PhantomData<O>,
}

impl<I: Divisible, WF: Fn(I, usize) -> I + Sync> ActivatedInput<I, I, WF, fn(I) -> I> {
    pub fn map<O: Send, MF: Fn(I) -> O + Sync>(
        self,
        map_function: MF,
    ) -> ActivatedInput<I, O, WF, MF> {
        ActivatedInput {
            input: self.input,
            work_function: self.work_function,
            map_function,
            output_type: PhantomData,
        }
    }
}

impl<I: Divisible, O: Send, WF: Fn(I, usize) -> I + Sync, MF: Fn(I) -> O + Sync> IntoIterator
    for ActivatedInput<I, O, WF, MF>
{
    type Item = O;
    type IntoIter = std::collections::linked_list::IntoIter<O>;
    fn into_iter(self) -> Self::IntoIter {
        let (input, work_function, map_function) =
            (self.input, self.work_function, self.map_function);
        let sequential_limit = (input.len() as f64).log(2.0).ceil() as usize;
        let outputs_list = schedule(
            input,
            &work_function,
            &|input| {
                let mut l = LinkedList::new();
                l.push_back((map_function)(input));
                l
            },
            &|mut left, mut right| {
                left.append(&mut right);
                left
            },
            Policy::Adaptive(sequential_limit),
        );
        outputs_list.into_iter()
    }
}

impl<I: Divisible, O: Send, WF: Fn(I, usize) -> I + Sync, MF: Fn(I) -> O + Sync>
    ActivatedInput<I, O, WF, MF>
{
    pub fn reduce<RF: Fn(O, O) -> O + Sync>(self, reduce_function: RF, policy: Policy) -> O {
        schedule(
            self.input,
            &self.work_function,
            &self.map_function,
            &reduce_function,
            policy,
        )
    }
}

// TODO: why on earth do I need Sync on I ?
impl<
        I: DivisibleAtIndex + Sync,
        O: Send + Sync,
        WF: Fn(I, usize) -> I + Sync,
        MF: Fn(I) -> O + Sync,
    > ActivatedInput<I, O, WF, MF>
{
    pub fn by_blocks(self, blocks_size: usize) -> impl Iterator<Item = O> {
        let (input, work_function, map_function) =
            (self.input, self.work_function, self.map_function);
        input.chunks(repeat(blocks_size)).flat_map(move |input| {
            let sequential_limit = (input.len() as f64).log(2.0).ceil() as usize;
            let outputs_list = schedule(
                input,
                &work_function,
                &|input| {
                    let mut l = LinkedList::new();
                    l.push_back((map_function)(input));
                    l
                },
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

pub trait Divisible: Sized + Send {
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Return our length.
    fn len(&self) -> usize;
    fn work<WF: Fn(Self, usize) -> Self + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<Self, Self, WF, fn(Self) -> Self> {
        ActivatedInput {
            input: self,
            work_function,
            map_function: |i| i,
            output_type: PhantomData,
        }
    }
    /// Easy api when we return no results.
    fn for_each<WF>(self, work_function: WF, policy: Policy)
    where
        WF: Fn(Self, usize) -> Self + Sync,
    {
        schedule(self, &work_function, &|_| (), &|_, _| (), policy)
    }
}

/// Some genericity to use only one scheduler for all different input types.
struct LocalWork<I: DivisibleAtIndex, O> {
    remaining_work: I,
    output: Option<O>,
}

impl<I: DivisibleAtIndex, O: Send> Divisible for LocalWork<I, O> {
    fn split(self) -> (Self, Self) {
        let (left_work, right_work) = self.remaining_work.split();
        (
            LocalWork {
                remaining_work: left_work,
                output: self.output,
            },
            LocalWork {
                remaining_work: right_work,
                output: None,
            },
        )
    }
    fn len(&self) -> usize {
        self.remaining_work.len()
    }
}

pub struct Chunks<I: DivisibleAtIndex, S: Iterator<Item = usize>> {
    remaining: I,
    remaining_sizes: S,
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
            mem::drop(self);
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
    fn map_reduce<MF, RF, O>(self, map_function: MF, reduce_function: RF) -> O
    where
        MF: Fn(Self) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send,
    {
        let full_work = LocalWork {
            remaining_work: self,
            output: None,
        };
        schedule(
            full_work,
            &|w, limit| -> LocalWork<Self, O> {
                let (todo_now, remaining) = w.remaining_work.split_at(limit);
                let new_result = map_function(todo_now);

                LocalWork {
                    remaining_work: remaining,
                    output: if let Some(output) = w.output {
                        //TODO: force sequential ? Some(output.fuse_with_policy(new_result, Policy::Sequential))
                        Some(reduce_function(output, new_result))
                    } else {
                        Some(new_result)
                    },
                }
            },
            &|w| w.output.unwrap(),
            &|left, right| reduce_function(left, right),
            Policy::Adaptive(1000),
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
