//! This module contains all traits enabling us to express some parallelism.
use scheduling::{schedule, Policy};
use std;
use std::collections::LinkedList;
use std::marker::PhantomData;

pub struct ActivatedInput<
    I: Divisible,
    O: Send,
    WF: Fn(I, usize) -> I + Sync,
    OF: Fn(I) -> O + Sync,
> {
    input: I,
    work_function: WF,
    output_function: OF, // TODO: rename to map
    output_type: PhantomData<O>,
}

impl<I: Divisible, WF: Fn(I, usize) -> I + Sync> ActivatedInput<I, I, WF, fn(I) -> I> {
    pub fn map<O: Send, OF: Fn(I) -> O + Sync>(
        self,
        map_function: OF,
    ) -> ActivatedInput<I, O, WF, OF> {
        ActivatedInput {
            input: self.input,
            work_function: self.work_function,
            output_function: map_function,
            output_type: PhantomData,
        }
    }
}

impl<I: Divisible, O: Send, WF: Fn(I, usize) -> I + Sync, OF: Fn(I) -> O + Sync> IntoIterator
    for ActivatedInput<I, O, WF, OF>
{
    type Item = O;
    type IntoIter = std::collections::linked_list::IntoIter<O>;
    fn into_iter(self) -> Self::IntoIter {
        let (input, work_function, output_function) =
            (self.input, self.work_function, self.output_function);
        let sequential_limit = (input.len() as f64).log(2.0).ceil() as usize;
        let outputs_list = schedule(
            input,
            &work_function,
            &|input| {
                let mut l = LinkedList::new();
                l.push_back((output_function)(input));
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

impl<I: Divisible, O: Send, WF: Fn(I, usize) -> I + Sync, OF: Fn(I) -> O + Sync>
    ActivatedInput<I, O, WF, OF>
{
    pub fn reduce<MF: Fn(O, O) -> O + Sync>(self, merge_function: MF, policy: Policy) -> O {
        schedule(
            self.input,
            &self.work_function,
            &self.output_function,
            &merge_function,
            policy,
        )
    }
}

// TODO: why on earth do I need Sync on I ?
impl<
        I: DivisibleAtIndex + Sync,
        O: Send + Sync,
        WF: Fn(I, usize) -> I + Sync,
        OF: Fn(I) -> O + Sync,
    > ActivatedInput<I, O, WF, OF>
{
    pub fn fold_with_blocks<B, F: FnMut(B, O) -> B>(
        self,
        init: B,
        mut f: F,
        blocks_size: usize,
        policy: Policy,
    ) -> B {
        let (mut input, work_function, output_function) =
            (self.input, self.work_function, self.output_function);

        //note: we cannot fold (adaptive) since folding consumes all functions
        let mut output = init;
        while input.len() > 0 {
            let size = std::cmp::min(blocks_size, input.len());
            let (head_block, remaining_input) = input.split_at(size);

            let outputs_list = schedule(
                head_block,
                &work_function,
                &|input| {
                    let mut l = LinkedList::new();
                    l.push_back((output_function)(input));
                    l
                },
                &|mut left, mut right| {
                    left.append(&mut right);
                    left
                },
                policy,
            );
            //note: we cannot fold (sequential) since folding consumes f
            for output_fragment in outputs_list {
                output = f(output, output_fragment);
            }
            input = remaining_input;
        }
        output
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
            output_function: |i| i,
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

pub trait DivisibleAtIndex: Divisible {
    /// Divide ourselves where requested.
    fn split_at(self, index: usize) -> (Self, Self);
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
