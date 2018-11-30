use activated_input::ActivatedInput;
use chunks::Chunks;
/// All scheduling available scheduling policies.
use folders::{fold::Fold, work_fold::WorkFold, Folder};
use scheduling::schedule;
use std::marker::PhantomData;
use {Divisible, DivisibleAtIndex};

#[derive(Copy, Clone)]
pub enum Policy {
    /// Adaptive scheduling policy with logarithmic block size.
    DefaultPolicy,
    /// Do all computations sequentially.
    Sequential,
    /// Recursively cut in two with join until given block size.
    Join(usize),
    /// Recursively cut in two with join_context until given block size.
    JoinContext(usize),
    /// Recursively cut in two with depjoin until given block size.
    DepJoin(usize),
    /// Advance locally with increasing block sizes. When stolen create tasks
    /// We need an initial block size.
    Adaptive(usize),
}

impl Default for Policy {
    fn default() -> Self {
        Policy::DefaultPolicy
    }
}

pub struct ParametrizedInput<I: Divisible> {
    pub(crate) input: I,
    pub(crate) policy: Policy,
}

impl<I: Divisible> ParametrizedInput<I> {
    pub fn work<WF: Fn(I, usize) -> I + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<WorkFold<I, WF>> {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self.input,
            folder,
            policy: self.policy,
        }
    }
    pub fn fold<O, ID, F>(self, identity: ID, fold_op: F) -> ActivatedInput<Fold<I, O, ID, F>>
    where
        O: Send + Sync,
        ID: Fn() -> O + Sync,
        F: Fn(O, I, usize) -> (O, I) + Sync,
    {
        let folder = Fold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        };
        ActivatedInput {
            input: self.input,
            folder,
            policy: self.policy,
        }
    }
    /// Easy api when we return no results.
    pub fn for_each<WF>(self, work_function: WF)
    where
        WF: Fn(I, usize) -> I + Sync,
    {
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }.map(|_| ());
        let reduce = |_, _| ();
        schedule(self.input, &folder, &reduce, self.policy)
    }
}

impl<I: DivisibleAtIndex> ParametrizedInput<I> {
    /// Get a sequential iterator on chunks of Self of given sizes.
    pub fn chunks<S: Iterator<Item = usize>>(self, sizes: S) -> Chunks<I, S> {
        Chunks {
            remaining: self.input,
            remaining_sizes: sizes,
        }
    }
    /// Easy api but use only when splitting generates no tangible work overhead.
    pub fn map_reduce<MF, RF, O>(self, map_function: MF, reduce_function: RF) -> O
    where
        MF: Fn(I) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send + Sync,
    {
        let folder = Fold {
            identity_op: || None,
            fold_op: |o: Option<O>, i: I, limit: usize| -> (Option<O>, I) {
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
            self.input,
            &folder,
            &|left, right| reduce_function(left, right),
            self.policy,
        )
    }
}
