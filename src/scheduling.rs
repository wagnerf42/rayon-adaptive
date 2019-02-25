//! Let factorize a huge amount of scheduling policies into one api.
use crate::atomiclist::{AtomicLink, AtomicList};
use crate::depjoin;
use crate::folders::Folder;
use crate::prelude::*;
use crate::smallchannel::{small_channel, SmallSender};
use crate::traits::Divisible;
use crate::utils::powers;
use crate::Policy;
use rayon::{current_num_threads, Scope};
#[cfg(feature = "logs")]
use rayon_logs::sequential_task;
use std::cell::RefCell;
use std::cmp::min;
use std::iter::once;
use std::iter::repeat;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};

// we use this boolean to prevent fine grain parallelism when coarse grain
// parallelism is still available in composed algorithms.
thread_local!(static SEQUENCE: RefCell<bool> = RefCell::new(false));

/// by default, min block size is log(n)
fn default_min_block_size(n: usize) -> usize {
    let power = ((n as f64 / (n as f64).log(2.0) + 1.0).log(2.0) - 1.0).floor();
    ((n as f64) / (2.0f64.powi(power as i32 + 1) - 1.0)).ceil() as usize
}

/// by default, max block size is sqrt(n)
fn default_max_block_size(n: usize) -> usize {
    ((n as f64).sqrt() * 10.0f64).ceil() as usize
}

/// compute a block size with the given function.
/// this allows us to ensure we enforce important bounds on sizes.
fn compute_size<F: Fn(usize) -> usize>(n: usize, sizing_function: F) -> usize {
    let p = current_num_threads();
    std::cmp::max(min(n / (2 * p), sizing_function(n)), 1)
}

pub(crate) fn schedule<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    policy: Policy,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    SEQUENCE.with(|s| {
        if *s.borrow() || input.base_length() == 1 {
            schedule_sequential(input, folder)
        } else {
            let block_size = match policy {
                Policy::Sequential => input.base_length(),
                Policy::DefaultPolicy => compute_size(input.base_length(), default_min_block_size),
                Policy::Join(block_size)
                | Policy::JoinContext(block_size)
                | Policy::DepJoin(block_size)
                | Policy::Adaptive(block_size, _) => block_size,
                Policy::Rayon => 1,
            };
            match policy {
                Policy::Sequential => schedule_sequential(input, folder),
                Policy::Join(_) => schedule_join(input, folder, reduce_function, block_size),
                Policy::JoinContext(_) => {
                    schedule_join_context(input, folder, reduce_function, block_size)
                }
                Policy::DepJoin(_) => schedule_depjoin(input, folder, reduce_function, block_size),
                Policy::Adaptive(min, max) => schedule_adaptive(
                    input,
                    folder.identity(),
                    folder,
                    reduce_function,
                    (|_| min, |_| max),
                ),
                Policy::DefaultPolicy => {
                    if block_size * 2 * current_num_threads() >= input.base_length() //TODO ASK should I call schedule_adaptive in this case?
                || (current_num_threads() as f64).log2() * (50.0f64)
                    >= (input.base_length() as f64) / (block_size as f64)
                    {
                        let max_size = compute_size(input.base_length(), default_max_block_size);
                        schedule_join_context_max_size(
                            input,
                            folder,
                            reduce_function,
                            block_size,
                            max_size,
                        )
                    } else {
                        let max_size = compute_size(input.base_length(), default_max_block_size);
                        schedule_adaptive(
                            input,
                            folder.identity(),
                            folder,
                            reduce_function,
                            (|_| block_size, |_| max_size),
                        )
                    }
                }
                Policy::Rayon => schedule_rayon_join_context(
                    input,
                    folder,
                    reduce_function,
                    rayon::current_num_threads(),
                ),
            }
        }
    })
}

fn schedule_sequential<F: Folder>(input: F::Input, folder: &F) -> F::Output {
    let len = input.base_length();
    let (io, i) = folder.fold(folder.identity(), input, len);
    folder.to_output(io, i)
}

fn schedule_join<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    block_size: usize,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    let len = input.base_length();
    if len <= block_size {
        schedule_sequential(input, folder)
    } else {
        let (i1, i2) = input.divide();
        let (r1, r2) = rayon::join(
            || schedule_join(i1, folder, reduce_function, block_size),
            || schedule_join(i2, folder, reduce_function, block_size),
        );
        reduce_function(r1, r2)
    }
}

fn schedule_join_context<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    block_size: usize,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    let len = input.base_length();
    if len <= block_size {
        schedule_sequential(input, folder)
    } else {
        let (i1, i2) = input.divide();
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context(i1, folder, reduce_function, block_size),
            |c| {
                if c.migrated() {
                    schedule_join_context(i2, folder, reduce_function, block_size)
                } else {
                    schedule_sequential(i2, folder)
                }
            },
        );
        reduce_function(r1, r2)
    }
}

fn schedule_rayon_join_context<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    split_limit: usize,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    if split_limit == 0 || input.base_length() == 1 {
        schedule_sequential(input, folder)
    } else {
        let (i1, i2) = input.divide();
        let (r1, r2) = rayon::join_context(
            |_| schedule_rayon_join_context(i1, folder, reduce_function, split_limit / 2),
            |c| {
                if c.migrated() {
                    schedule_rayon_join_context(
                        i2,
                        folder,
                        reduce_function,
                        rayon::current_num_threads() * 2,
                    )
                } else {
                    schedule_rayon_join_context(i2, folder, reduce_function, split_limit / 2)
                }
            },
        );
        reduce_function(r1, r2)
    }
}

fn schedule_join_context_max_size<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    min_size: usize,
    max_size: usize,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    let len = input.base_length();
    if len <= min_size {
        schedule_sequential(input, folder)
    } else {
        let (i1, i2) = input.divide();
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context_max_size(i1, folder, reduce_function, min_size, max_size),
            |c| {
                if len > max_size || c.migrated() {
                    schedule_join_context_max_size(i2, folder, reduce_function, min_size, max_size)
                } else {
                    SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
                    let sequential_output = schedule_sequential(i2, folder);
                    SEQUENCE.with(|s| *s.borrow_mut() = false);
                    sequential_output
                }
            },
        );
        reduce_function(r1, r2)
    }
}

fn schedule_depjoin<F, RF>(
    input: F::Input,
    folder: &F,
    reduce_function: &RF,
    block_size: usize,
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
{
    let len = input.base_length();
    if len <= block_size {
        schedule_sequential(input, folder)
    } else {
        let (i1, i2) = input.divide();
        depjoin(
            || schedule_depjoin(i1, folder, reduce_function, block_size),
            || schedule_depjoin(i2, folder, reduce_function, block_size),
            reduce_function,
        )
    }
}

struct AdaptiveWorker<
    'a,
    'b,
    F: Folder + 'b,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync + 'b,
    MINSIZE: Fn(usize) -> usize + Send + Copy,
    MAXSIZE: Fn(usize) -> usize + Send + Copy,
> {
    input: F::Input,
    partial_output: F::IntermediateOutput,
    block_sizes: (MINSIZE, MAXSIZE),
    min_block_size: usize,
    max_block_size: usize,
    stolen: &'a AtomicBool,
    sender: SmallSender<F::Input>,
    folder: &'b F,
    reduce_function: &'b RF,
    phantom: PhantomData<(F::Output)>,
}

impl<'a, 'b, F, RF, MINSIZE, MAXSIZE> AdaptiveWorker<'a, 'b, F, RF, MINSIZE, MAXSIZE>
where
    F: Folder + 'b,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync + 'b,
    MINSIZE: Fn(usize) -> usize + Send + Copy,
    MAXSIZE: Fn(usize) -> usize + Send + Copy,
{
    fn new(
        input: F::Input,
        partial_output: F::IntermediateOutput,
        block_sizes: (MINSIZE, MAXSIZE),
        stolen: &'a AtomicBool,
        sender: SmallSender<F::Input>,
        folder: &'b F,
        reduce_function: &'b RF,
    ) -> Self {
        let min_block_size = compute_size(input.base_length(), block_sizes.0);
        let max_block_size = compute_size(input.base_length(), block_sizes.1);

        AdaptiveWorker {
            input,
            partial_output,
            block_sizes,
            min_block_size,
            max_block_size,
            stolen,
            sender,
            folder,
            reduce_function,
            phantom: PhantomData,
        }
    }
    //    fn is_stolen(&self) -> bool {
    //        self.stolen.load(Ordering::Relaxed)
    //    }
    //    fn answer_steal(self) -> F::Output {
    //        let (my_half, his_half) = self.input.divide();
    //        if his_half.base_length() != 0 {
    //            self.sender.send(his_half);
    //        }
    //        schedule_adaptive(
    //            my_half,
    //            self.partial_output,
    //            self.folder,
    //            self.reduce_function,
    //            self.block_sizes,
    //        )
    //    }

    fn schedule(self) -> F::Output {
        // TODO: automate this min everywhere ?
        // TODO: factorize a little bit
        // start by computing a little bit in order to get a first output
        let partial_output = self.partial_output;
        let remaining_input = self.input;
        let stolen_bool = self.stolen;
        let folder = self.folder;
        let max_size = self.max_block_size;
        match powers(self.min_block_size)
            .take_while(|&size| size < max_size)
            .chain(repeat(max_size))
            .take_while(|_| !stolen_bool.load(Ordering::Relaxed))
            .try_fold(
                (partial_output, remaining_input),
                |(output, input), size| {
                    let checked_size = min(input.base_length(), size); //TODO: remove all these mins
                    if checked_size > 0 {
                        Ok(folder.fold(output, input, checked_size))
                    } else {
                        Err(folder.to_output(output, input))
                    }
                },
            ) {
            Ok((mut output, mut remaining_input)) => {
                let remaining_length = remaining_input.base_length();
                if remaining_length > self.min_block_size {
                    let (my_half, his_half) = remaining_input.divide();
                    if his_half.base_length() > 0 {
                        self.sender.send(his_half);
                    }
                    schedule_adaptive(
                        my_half,
                        output,
                        self.folder,
                        self.reduce_function,
                        self.block_sizes,
                    )
                } else {
                    if remaining_length != 0 {
                        let final_result = folder.fold(output, remaining_input, remaining_length);
                        output = final_result.0;
                        remaining_input = final_result.1;
                    }
                    self.folder.to_output(output, remaining_input)
                }
            }
            Err(output) => output,
        }
    }
}

fn schedule_adaptive<F, RF, MINSIZE, MAXSIZE>(
    input: F::Input,
    partial_output: F::IntermediateOutput,
    folder: &F,
    reduce_function: &RF,
    block_sizes: (MINSIZE, MAXSIZE),
) -> F::Output
where
    F: Folder,
    RF: Fn(F::Output, F::Output) -> F::Output + Sync,
    MINSIZE: Fn(usize) -> usize + Send + Copy,
    MAXSIZE: Fn(usize) -> usize + Send + Copy,
{
    let size = input.base_length();
    if size <= compute_size(size, block_sizes.0) {
        let (io, i) = folder.fold(partial_output, input, size);
        folder.to_output(io, i)
    } else {
        let stolen = &AtomicBool::new(false);
        let (sender, receiver) = small_channel();

        let worker = AdaptiveWorker::new(
            input,
            partial_output,
            block_sizes,
            stolen,
            sender,
            folder,
            reduce_function,
        );

        //TODO depjoin instead of join
        let (o1, maybe_o2) = rayon::join(
            move || worker.schedule(),
            move || {
                stolen.store(true, Ordering::Relaxed);
                let input: F::Input;
                #[cfg(feature = "logs")]
                {
                    let option = sequential_task("waiting", 1, || receiver.recv());
                    input = option?;
                }
                #[cfg(not(feature = "logs"))]
                {
                    input = receiver.recv()?;
                }
                assert!(input.base_length() > 0);
                Some(schedule_adaptive(
                    input,
                    folder.identity(),
                    folder,
                    reduce_function,
                    block_sizes,
                ))
            },
        );

        let fusion_needed = maybe_o2.is_some();
        if fusion_needed {
            reduce_function(o1, maybe_o2.unwrap())
        } else {
            o1
        }
    }
}

/******************** fully adaptive scheduling ***********************/

/// We are going to do one big fold operation in order to compute
/// the final result.
/// Sometimes we fold on some input but sometimes we also fold
/// on intermediate outputs.
/// Having an enumerated type enables to conveniently iterate on both types.
enum FoldElement<I, O2> {
    Input(I),
    Output(O2),
}

pub(crate) fn fold_with_help<F, O1, FOLD1, RET, S>(
    input: F::Input,
    o1: O1,
    fold1: FOLD1,
    slave_folder: &F,
    retrieve: RET,
    sizes: S,
    policy: Policy,
) -> O1
where
    F: Folder + Send,
    O1: Send,
    F::Input: DivisibleIntoBlocks,
    FOLD1: Fn(O1, F::Input, usize) -> (O1, F::Input) + Sync,
    RET: Fn(O1, F::Output) -> O1 + Sync,
    S: Iterator<Item = usize> + Send,
{
    let (min_size, max_size) = match policy {
        Policy::Adaptive(min_size, max_size) => (min_size, max_size),
        Policy::DefaultPolicy => (
            compute_size(input.base_length(), default_min_block_size),
            compute_size(input.base_length(), default_max_block_size),
        ),
        _ => panic!("for now only adaptive or default policies for help"),
    };
    let input_length = input.base_length();
    let stolen_stuffs: &AtomicList<(Option<F::Output>, Option<F::Input>)> = &AtomicList::new();
    let completed_sizes = sizes.chain(once(input_length));
    rayon::scope(|s| {
        input
            .chunks(completed_sizes)
            .flat_map(|chunk| {
                once(FoldElement::Input(chunk)).chain(stolen_stuffs.iter().flat_map(|(o2, i)| {
                    o2.map(FoldElement::Output)
                        .into_iter()
                        .chain(i.map(FoldElement::Input).into_iter())
                }))
            })
            .fold(o1, |o1, element| match element {
                FoldElement::Input(i) => master_work(
                    s,
                    o1,
                    i,
                    &fold1,
                    slave_folder,
                    stolen_stuffs,
                    min_size,
                    max_size,
                ),
                FoldElement::Output(o2) => retrieve(o1, o2),
            })
    })
}

fn spawn_stealing_task<'scope, F>(
    scope: &Scope<'scope>,
    slave_folder: &'scope F,
    min_size: usize,
    max_size: usize,
) -> SmallSender<AtomicLink<(Option<F::Output>, Option<F::Input>)>>
where
    F: Folder + 'scope + Send,
    F::Input: DivisibleIntoBlocks + 'scope,
{
    let (sender, receiver) = small_channel();
    scope.spawn(move |s| {
        let stolen_input: Option<AtomicLink<(Option<F::Output>, Option<F::Input>)>>;
        #[cfg(feature = "logs")]
        {
            stolen_input = rayon_logs::sequential_task("slave wait", 1, || receiver.recv());
        }
        #[cfg(not(feature = "logs"))]
        {
            stolen_input = receiver.recv();
        }
        if stolen_input.is_none() {
            return;
        }
        slave_work(s, stolen_input.unwrap(), slave_folder, min_size, max_size)
    });
    sender
}

fn master_work<'scope, F, O1, FOLD1>(
    scope: &Scope<'scope>,
    init: O1,
    input: F::Input,
    fold: &FOLD1,
    slave_folder: &'scope F,
    stolen_stuffs: &AtomicList<(Option<F::Output>, Option<F::Input>)>,
    min_size: usize,
    max_size: usize,
) -> O1
where
    F: Folder + 'scope + Send,
    F::Input: DivisibleIntoBlocks + 'scope,
    O1: Send,
    FOLD1: Fn(O1, F::Input, usize) -> (O1, F::Input),
{
    let mut input = input;
    let mut current_output = init;
    loop {
        let sender = spawn_stealing_task(scope, slave_folder, min_size, max_size);
        // let's work sequentially until stolen
        match powers(min_size)
            .take_while(|&p| p < max_size)
            .chain(repeat(max_size))
            .take_while(|_| !sender.receiver_is_waiting())
            .try_fold((current_output, input), |(output, input), size| {
                let checked_size = min(input.base_length(), size); //TODO: remove all these mins
                if checked_size > 0 {
                    Ok(fold(output, input, checked_size))
                } else {
                    Err(output)
                }
            }) {
            Ok((output, remaining_input)) => {
                if remaining_input.base_length() > min_size {
                    let (my_half, his_half) = remaining_input.divide();
                    if his_half.base_length() > 0 {
                        let stolen_node = stolen_stuffs.push_front((None, Some(his_half)));
                        sender.send(stolen_node);
                    }
                    input = my_half;
                    current_output = output;
                } else {
                    let length = remaining_input.base_length();
                    return fold(output, remaining_input, length).0;
                }
            }
            Err(output) => return output,
        }
    }
}

//TODO: we could maybe avoid code duplication between master and slave with a dummy head of the
//list for the master
fn slave_work<'scope, F>(
    scope: &Scope<'scope>,
    node: AtomicLink<(Option<F::Output>, Option<F::Input>)>,
    slave_folder: &'scope F,
    min_size: usize,
    max_size: usize,
) where
    F: Folder + 'scope + Send,
    F::Input: DivisibleIntoBlocks + 'scope,
{
    let mut input = node.take().unwrap().1.unwrap();
    let mut o2 = slave_folder.identity();
    loop {
        let sender = spawn_stealing_task(scope, slave_folder, min_size, max_size);
        // let's work sequentially until stolen
        match powers(min_size)
            .take_while(|&p| p < max_size)
            .chain(repeat(max_size))
            .take_while(|_| !sender.receiver_is_waiting() && !node.requested())
            .try_fold((o2, input), |(output2, input), size| {
                let checked_size = min(input.base_length(), size); //TODO: remove all these mins
                if checked_size > 0 {
                    Ok(slave_folder.fold(output2, input, checked_size))
                } else {
                    Err((output2, input))
                }
            }) {
            Ok((output2, remaining_input)) => {
                if node.requested() {
                    // retrieval operations are prioritized over steal ops
                    let (completed, remaining_input) = remaining_input.divide_at(0);
                    node.replace((
                        Some(slave_folder.to_output(output2, completed)),
                        Some(remaining_input),
                    ));
                    return;
                } else {
                    // check if enough is left
                    let length = remaining_input.base_length();
                    if length > min_size {
                        let (my_half, his_half) = remaining_input.divide();
                        // TODO: have an empty method
                        if his_half.base_length() > 0 {
                            let stolen_node = (&node).split((None, Some(his_half)));
                            sender.send(stolen_node)
                        }
                        input = my_half;
                        o2 = output2;
                    } else {
                        // just fold it locally
                        let (intermediate_output, input) =
                            slave_folder.fold(output2, remaining_input, length);
                        node.replace((
                            Some(slave_folder.to_output(intermediate_output, input)),
                            None,
                        ));
                        return;
                    }
                }
            }
            Err((output2, input)) => {
                node.replace((Some(slave_folder.to_output(output2, input)), None));
                return;
            }
        }
    }
}
