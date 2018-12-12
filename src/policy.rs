use crate::activated_input::ActivatedInput;
/// All scheduling available scheduling policies.
use crate::folders::{fold::Fold, work_fold::WorkFold, Folder};
use crate::scheduling::schedule;
use std::marker::PhantomData;
use crate::{Divisible, DivisibleIntoBlocks};

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

pub trait AdaptiveRunner<I: Divisible>: Sized {
    fn input_len(&self) -> usize;
    /// Return input and scheduling policy.
    fn input_and_policy(self) -> (I, Policy);
    fn work<WF: Fn(I, usize) -> I + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<WorkFold<I, WF>> {
        let (input, policy) = self.input_and_policy();
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        };
        ActivatedInput {
            input,
            folder,
            policy,
        }
    }
    fn partial_fold<O, ID, F>(self, identity: ID, fold_op: F) -> ActivatedInput<Fold<I, O, ID, F>>
    where
        O: Send + Sync,
        ID: Fn() -> O + Sync,
        F: Fn(O, I, usize) -> (O, I) + Sync,
    {
        let (input, policy) = self.input_and_policy();
        let folder = Fold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        };
        ActivatedInput {
            input,
            folder,
            policy,
        }
    }
    /// Easy api when we return no results.
    fn partial_for_each<WF>(self, work_function: WF)
    where
        WF: Fn(I, usize) -> I + Sync,
    {
        let (input, policy) = self.input_and_policy();
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }
        .map(|_| ());
        let reduce = |_, _| ();
        schedule(input, &folder, &reduce, policy)
    }
}

pub struct ParametrizedInput<I: Divisible> {
    pub(crate) input: I,
    pub(crate) policy: Policy,
}

impl<I: Divisible> AdaptiveRunner<I> for ParametrizedInput<I> {
    fn input_len(&self) -> usize {
        self.input.base_length()
    }
    fn input_and_policy(self) -> (I, Policy) {
        (self.input, self.policy)
    }
}

impl<I: Divisible> AdaptiveRunner<I> for I {
    fn input_len(&self) -> usize {
        self.base_length()
    }
    fn input_and_policy(self) -> (I, Policy) {
        (self, Default::default())
    }
}

pub trait BlockAdaptiveRunner<I: DivisibleIntoBlocks>: AdaptiveRunner<I> {
    /// Easy api but use only when splitting generates no tangible work overhead.
    fn map_reduce<MF, RF, O>(self, map_function: MF, reduce_function: RF) -> O
    where
        MF: Fn(I) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send + Sync,
    {
        let (input, policy) = self.input_and_policy();
        let folder = Fold {
            identity_op: || None,
            fold_op: |o: Option<O>, i: I, limit: usize| -> (Option<O>, I) {
                let (todo_now, remaining) = i.divide_at(limit);
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
        }
        .map(|o| o.unwrap());

        schedule(
            input,
            &folder,
            &|left, right| reduce_function(left, right),
            policy,
        )
    }
}

impl<I: DivisibleIntoBlocks> BlockAdaptiveRunner<I> for ParametrizedInput<I> {}
impl<I: DivisibleIntoBlocks> BlockAdaptiveRunner<I> for I {}
