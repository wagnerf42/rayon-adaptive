//! the folded stuff, ready to be reduced.
use folders::Map;
use scheduling::schedule;
use std::collections::linked_list;
use std::collections::linked_list::LinkedList;
use {DivisibleAtIndex, Folder, Policy};

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

impl<I: DivisibleAtIndex, F: Folder<Input = I>> ActivatedInput<F> {
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
