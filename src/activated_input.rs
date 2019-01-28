//! the folded stuff, ready to be reduced.
use crate::folders::Map;
use crate::prelude::*;
use crate::scheduling::{fold_with_help, schedule};
use crate::{DivisibleIntoBlocks, Folder, Policy};
use std::collections::linked_list;
use std::collections::linked_list::LinkedList;

/// Lazily store everything for folding.
pub struct ActivatedInput<F: Folder> {
    pub(crate) input: F::Input, // what we fold
    pub(crate) folder: F,       // how we fold it
    pub(crate) policy: Policy,  // with what scheduler
}

impl<F: Folder> IntoIterator for ActivatedInput<F> {
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

impl<F: Folder> ActivatedInput<F> {
    pub fn map<O: Send + Sync, M: Fn(F::Output) -> O + Sync>(
        self,
        map_op: M,
    ) -> ActivatedInput<Map<F, O, M>> {
        ActivatedInput {
            input: self.input,
            folder: self.folder.map(map_op),
            policy: self.policy,
        }
    }
    pub fn reduce<RF: Fn(F::Output, F::Output) -> F::Output + Sync>(
        self,
        reduce_function: RF,
    ) -> F::Output {
        let (input, folder, policy) = (self.input, self.folder, self.policy);
        schedule(input, &folder, &reduce_function, policy)
    }
}

impl<I: AdaptiveIterator + DivisibleIntoBlocks, F: Folder<Input = I> + Send> ActivatedInput<F> {
    pub fn helping_for_each<FOREACH, RET>(self, f: FOREACH, retrieve: RET)
    where
        FOREACH: Fn(I::Item) + Sync,
        RET: Fn(F::Output) + Sync,
    {
        let (input, folder) = (self.input, self.folder);
        let f_ref = &f;
        let master_fold = |_: (), i: I, size: usize| -> ((), I) {
            let (todo, remaining) = i.divide_at(size);
            todo.into_iter().for_each(f_ref);
            ((), remaining)
        };
        let master_retrieve = |_, v| retrieve(v);
        fold_with_help(input, (), master_fold, &folder, master_retrieve)
    }
}

impl<I: DivisibleIntoBlocks, F: Folder<Input = I> + Send> ActivatedInput<F> {
    pub fn helping_partial_fold<B, FOLD, RET>(self, init: B, f: FOLD, retrieve: RET) -> B
    where
        B: Send,
        FOLD: Fn(B, I, usize) -> (B, I) + Sync,
        RET: Fn(B, F::Output) -> B + Sync,
    {
        let (input, folder) = (self.input, self.folder);
        fold_with_help(input, init, f, &folder, retrieve)
    }

    pub fn by_blocks<S: Iterator<Item = usize>>(
        self,
        blocks_sizes: S,
    ) -> impl Iterator<Item = F::Output> {
        let (input, folder, policy) = (self.input, self.folder, self.policy);

        let list_folder = folder.map(|o| {
            let mut l = LinkedList::new();
            l.push_back(o);
            l
        });

        input.chunks(blocks_sizes).flat_map(move |input| {
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
        })
    }
}
