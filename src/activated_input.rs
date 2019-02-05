//! the folded stuff, ready to be reduced.
use crate::folders::Map;
use crate::prelude::*;
use crate::scheduling::{fold_with_help, schedule};
use crate::traits::{BasicPower, BlockedOrMore};
use crate::{DivisibleIntoBlocks, Folder, Policy};
use std::cmp::min;
use std::collections::linked_list;
use std::collections::linked_list::LinkedList;
use std::iter::{once, Chain, Empty, Once};
use std::marker::PhantomData;

/// Lazily store everything for folding.
pub struct ActivatedInput<F: Folder, S, P> {
    pub(crate) input: F::Input,       // what we fold
    pub(crate) folder: F,             // how we fold it
    pub(crate) policy: Policy,        // with what scheduler
    pub(crate) sizes: S,              // blocks sizes iterator (if any)
    pub(crate) power: PhantomData<P>, // what can we do
}

impl<F: Folder> IntoIterator for ActivatedInput<F, Empty<usize>, BasicPower>
where
    F: Folder,
    F::Input: Divisible<Power = BasicPower>,
{
    type Item = F::Output;
    type IntoIter = linked_list::IntoIter<F::Output>;
    fn into_iter(self) -> Self::IntoIter {
        let (input, folder, policy) = (self.input, self.folder, self.policy);
        let list_folder = folder.map(|o| {
            let mut l = LinkedList::new();
            l.push_back(o);
            l
        });

        let outputs_list = schedule(
            input,
            &list_folder,
            &|mut left, mut right| {
                left.append(&mut right);
                left
            },
            policy,
        );
        outputs_list.into_iter()
    }
}

impl<F: Folder, S, P> ActivatedInput<F, S, P> {
    pub fn map<O: Send + Sync, M: Fn(F::Output) -> O + Sync>(
        self,
        map_op: M,
    ) -> ActivatedInput<Map<F, O, M>, S, P> {
        ActivatedInput {
            input: self.input,
            folder: self.folder.map(map_op),
            policy: self.policy,
            sizes: self.sizes,
            power: self.power,
        }
    }
}

impl<F> ActivatedInput<F, Empty<usize>, BasicPower>
where
    F: Folder,
    F::Input: Divisible<Power = BasicPower>,
{
    pub fn reduce<RF: Fn(F::Output, F::Output) -> F::Output + Sync>(
        self,
        reduce_function: RF,
    ) -> F::Output {
        let (input, folder, policy) = (self.input, self.folder, self.policy);
        schedule(input, &folder, &reduce_function, policy)
    }
}

impl<F, S> ActivatedInput<F, S, BlockedOrMore>
where
    F: Folder,
    F::Input: DivisibleIntoBlocks,
    S: Iterator<Item = usize>,
{
    pub fn reduce<RF: Fn(F::Output, F::Output) -> F::Output + Sync>(
        self,
        reduce_function: RF,
    ) -> F::Output {
        let (input, folder, policy, sizes) = (self.input, self.folder, self.policy, self.sizes);
        let reduce_ref = &reduce_function;
        let length = input.base_length();
        let mut outputs = input
            .chunks(sizes.chain(once(length)))
            .map(|input| schedule(input, &folder, reduce_ref, policy));
        let first_output = outputs.next().unwrap();
        outputs.fold(first_output, reduce_ref)
    }
}

pub struct OutputIterator<F: Folder, S> {
    remaining_input: F::Input,
    folder: Map<F, LinkedList<F::Output>, fn(F::Output) -> LinkedList<F::Output>>,
    sizes: S,
    policy: Policy,
    block_iterator: Option<linked_list::IntoIter<F::Output>>,
}

impl<F: Folder, S: Iterator<Item = usize>> OutputIterator<F, Chain<S, Once<usize>>> {
    fn new(input: F::Input, folder: F, policy: Policy, sizes: S) -> Self {
        fn into_list<T>(x: T) -> LinkedList<T> {
            let mut l = LinkedList::new();
            l.push_back(x);
            l
        }
        let length = input.base_length();

        OutputIterator {
            remaining_input: input,
            folder: folder.map(into_list),
            sizes: sizes.chain(once(length)),
            policy,
            block_iterator: None,
        }
    }
}

impl<F, S> Iterator for OutputIterator<F, S>
where
    F: Folder,
    F::Input: DivisibleIntoBlocks,
    S: Iterator<Item = usize>,
{
    type Item = F::Output;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iterator) = self.block_iterator.as_mut() {
            let possible_next = iterator.next();
            if possible_next.is_some() {
                return possible_next;
            }
        }
        if self.remaining_input.base_length() == 0 {
            None
        } else {
            let next_size = min(
                self.sizes.next().expect("not enough sizes for chunks"),
                self.remaining_input.base_length(),
            );
            let next_chunk = self.remaining_input.cut_left_at(next_size);
            let outputs_list = schedule(
                next_chunk,
                &self.folder,
                &|mut left, mut right| {
                    left.append(&mut right);
                    left
                },
                self.policy,
            );
            self.block_iterator = Some(outputs_list.into_iter());
            self.block_iterator.as_mut().unwrap().next()
        }
    }
}

impl<F, S> IntoIterator for ActivatedInput<F, S, BlockedOrMore>
where
    F: Folder,
    F::Input: DivisibleIntoBlocks,
    S: Iterator<Item = usize>,
{
    type Item = F::Output;
    type IntoIter = OutputIterator<F, Chain<S, Once<usize>>>;
    fn into_iter(self) -> Self::IntoIter {
        let (input, folder, policy, sizes) = (self.input, self.folder, self.policy, self.sizes);
        OutputIterator::new(input, folder, policy, sizes)
    }
}

//    pub fn by_blocks<S: Iterator<Item = usize>>(
//        self,
//        blocks_sizes: S,
//    ) -> impl Iterator<Item = F::Output> {
//        let (input, folder, policy) = (self.input, self.folder, self.policy);
//
//        let list_folder = folder.map(|o| {
//            let mut l = LinkedList::new();
//            l.push_back(o);
//            l
//        });
//
//        input.chunks(blocks_sizes).flat_map(move |input| {
//            let outputs_list = schedule(
//                input,
//                &list_folder,
//                &|mut left, mut right| {
//                    left.append(&mut right);
//                    left
//                },
//                policy,
//            );
//            outputs_list.into_iter()
//        })
//    }

impl<
        I: AdaptiveIterator + DivisibleIntoBlocks,
        F: Folder<Input = I> + Send,
        S: Iterator<Item = usize> + Send,
    > ActivatedInput<F, S, BlockedOrMore>
{
    pub fn helping_for_each<FOREACH, RET>(self, f: FOREACH, retrieve: RET)
    where
        FOREACH: Fn(I::Item) + Sync,
        RET: Fn(F::Output) + Sync,
    {
        let (input, folder, sizes) = (self.input, self.folder, self.sizes);
        let f_ref = &f;
        let master_fold = |_: (), i: I, size: usize| -> ((), I) {
            let (todo, remaining) = i.divide_at(size);
            todo.into_iter().for_each(f_ref);
            ((), remaining)
        };
        let master_retrieve = |_, v| retrieve(v);
        fold_with_help(input, (), master_fold, &folder, master_retrieve, sizes)
    }
}

impl<I: DivisibleIntoBlocks, F: Folder<Input = I> + Send, S: Iterator<Item = usize> + Send>
    ActivatedInput<F, S, BlockedOrMore>
{
    pub fn helping_partial_fold<B, FOLD, RET>(self, init: B, f: FOLD, retrieve: RET) -> B
    where
        B: Send,
        FOLD: Fn(B, I, usize) -> (B, I) + Sync,
        RET: Fn(B, F::Output) -> B + Sync,
    {
        let (input, folder, sizes) = (self.input, self.folder, self.sizes);
        fold_with_help(input, init, f, &folder, retrieve, sizes)
    }

    pub fn helping_cutting_fold<B, FOLD, RET>(self, init: B, f: FOLD, retrieve: RET) -> B
    where
        B: Send,
        FOLD: Fn(B, I) -> B + Sync,
        RET: Fn(B, F::Output) -> B + Sync,
    {
        let (input, folder, sizes) = (self.input, self.folder, self.sizes);
        let cutting_fold = |io, i: I, limit| {
            let (todo, remaining) = i.divide_at(limit);
            (f(io, todo), remaining)
        };
        fold_with_help(input, init, cutting_fold, &folder, retrieve, sizes)
    }
}
