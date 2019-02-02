use crate::activated_input::ActivatedInput;
/// All scheduling available scheduling policies.
use crate::folders::{fold::Fold, work_fold::WorkFold, Folder};
use crate::scheduling::schedule;
use crate::traits::{BasicPower, BlockedOrMore};
use crate::{Divisible, DivisibleIntoBlocks};
use std::iter::{empty, once, Empty};
use std::marker::PhantomData;

#[derive(Copy, Clone)]
pub enum Policy {
    /// Adaptive scheduling policy with dynamic block sizes.
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
    /// We need an initial block size and a maximal block size.
    Adaptive(usize, usize),
    /// Mirrors the rayon join context.
    Rayon,
}

impl Default for Policy {
    fn default() -> Self {
        Policy::DefaultPolicy
    }
}

/// We can assign a scheduling policy to any `Divisible input`.
/// We obtain this structure holding policy and input together.
pub struct ParametrizedInput<I: Divisible, S: Iterator<Item = usize>> {
    pub(crate) input: I,
    pub(crate) policy: Policy,
    pub(crate) sizes: S,
}

/********************************************************************************/
/*                          Runner Traits definitions                           */
/********************************************************************************/

/// Abstract between Input and ParametrizedInput in order to avoid duplicated code.
pub trait AdaptiveRunner<I: Divisible, S: Iterator<Item = usize>>: Sized {
    /// Return input's base length.
    /// Useful for computing blocks sizes.
    fn input_length(&self) -> usize;
    /// Return input, policy and sizes iterator.
    fn input_policy_sizes(self) -> (I, Policy, S);
}

/// The stuff everyone can do.
pub trait AllAdaptiveRunner<I: Divisible, S: Iterator<Item = usize>, P>:
    AdaptiveRunner<I, S>
{
    fn work<WF: Fn(I, usize) -> I + Sync>(
        self,
        work_function: WF,
    ) -> ActivatedInput<WorkFold<I, WF>, S, P> {
        let (input, policy, sizes) = self.input_policy_sizes();
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        };
        ActivatedInput {
            input,
            folder,
            policy,
            sizes,
            power: PhantomData,
        }
    }
    fn partial_fold<O, ID, F>(
        self,
        identity: ID,
        fold_op: F,
    ) -> ActivatedInput<Fold<I, O, ID, F>, S, P>
    where
        O: Send + Sync,
        ID: Fn() -> O + Sync,
        F: Fn(O, I, usize) -> (O, I) + Sync,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        let folder = Fold {
            identity_op: identity,
            fold_op,
            phantom: PhantomData,
        };
        ActivatedInput {
            input,
            folder,
            policy,
            sizes,
            power: PhantomData,
        }
    }

    /// Easy api when we return no results.
    /// This gets specialized as we move up the traits hierarchy.
    fn partial_for_each<WF>(self, work_function: WF)
    where
        WF: Fn(I, usize) -> I + Sync;
}

/// The stuff you can only do if at least DivisibleIntoBlocks
pub trait BlockAdaptiveRunner<I: DivisibleIntoBlocks, S: Iterator<Item = usize>>:
    AdaptiveRunner<I, S>
{
    /// Replace block sizes iterator (if any) by given one.
    fn by_blocks<S2: Iterator<Item = usize>>(self, sizes: S2) -> ParametrizedInput<I, S2> {
        let (input, policy, _) = self.input_policy_sizes();
        ParametrizedInput {
            input,
            policy,
            sizes,
        }
    }

    /// Easy api but use only when splitting generates no tangible work overhead.
    fn map_reduce<MF, RF, O>(self, map_function: MF, reduce_function: RF) -> O
    where
        MF: Fn(I) -> O + Sync,
        RF: Fn(O, O) -> O + Sync,
        O: Send + Sync,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
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

        let reduce_reference = &reduce_function;
        let folder_ref = &folder;

        let length = input.base_length();
        let mut outputs = input.chunks(sizes.chain(once(length))).map(|input| {
            schedule(
                input,
                folder_ref,
                &|left, right| reduce_reference(left, right),
                policy,
            )
        });
        let first_output = outputs.next().unwrap();
        outputs.fold(first_output, reduce_reference)
    }
}

/********************************************************************************/
/*                          Runner Traits implementations                       */
/********************************************************************************/

// Runner
impl<I: Divisible, S: Iterator<Item = usize>> AdaptiveRunner<I, S> for ParametrizedInput<I, S> {
    fn input_length(&self) -> usize {
        self.input.base_length()
    }
    fn input_policy_sizes(self) -> (I, Policy, S) {
        (self.input, self.policy, self.sizes)
    }
}

impl<I: Divisible> AdaptiveRunner<I, Empty<usize>> for I {
    fn input_length(&self) -> usize {
        self.base_length()
    }
    fn input_policy_sizes(self) -> (I, Policy, Empty<usize>) {
        (self, Default::default(), empty())
    }
}

// All
impl<I: Divisible<Power = BasicPower>, R: AdaptiveRunner<I, Empty<usize>>>
    AllAdaptiveRunner<I, Empty<usize>, BasicPower> for R
{
    /// Easy api when we return no results.
    fn partial_for_each<WF>(self, work_function: WF)
    where
        WF: Fn(I, usize) -> I + Sync,
    {
        let (input, policy, _) = self.input_policy_sizes();
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }
        .map(|_| ());
        let reduce = |_, _| ();
        schedule(input, &folder, &reduce, policy)
    }
}

impl<I: DivisibleIntoBlocks, S: Iterator<Item = usize>, R: AdaptiveRunner<I, S>>
    AllAdaptiveRunner<I, S, BlockedOrMore> for R
{
    /// Easy api when we return no results.
    fn partial_for_each<WF>(self, work_function: WF)
    where
        WF: Fn(I, usize) -> I + Sync,
    {
        let (input, policy, sizes) = self.input_policy_sizes();
        let folder = WorkFold {
            work_function,
            phantom: PhantomData,
        }
        .map(|_| ());
        let reduce = |_, _| ();

        for input in input.chunks(sizes) {
            schedule(input, &folder, &reduce, policy)
        }
    }
}

impl<I, S> BlockAdaptiveRunner<I, S> for ParametrizedInput<I, S>
where
    I: DivisibleIntoBlocks,
    S: Iterator<Item = usize>,
{
}

impl<I> BlockAdaptiveRunner<I, Empty<usize>> for I where I: DivisibleIntoBlocks {}
