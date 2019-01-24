//! Let factorize a huge amount of scheduling policies into one api.
use crate::depjoin;
use crate::folders::Folder;
use crate::smallchannel::{small_channel, SmallSender};
use crate::traits::Divisible;
use crate::Policy;
use rayon::current_num_threads;
#[cfg(feature = "logs")]
use rayon_logs::sequential_task;
use std::cell::RefCell;
use std::cmp::min;
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
    if split_limit == 0 {
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
    current_block_size: usize,
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
        let current_block_size = min_block_size;

        AdaptiveWorker {
            input,
            partial_output,
            block_sizes,
            current_block_size,
            min_block_size,
            max_block_size,
            stolen,
            sender,
            folder,
            reduce_function,
            phantom: PhantomData,
        }
    }
    fn is_stolen(&self) -> bool {
        self.stolen.load(Ordering::Relaxed)
    }
    fn answer_steal(self) -> F::Output {
        let (my_half, his_half) = self.input.divide();
        if his_half.base_length() != 0 {
            self.sender.send(his_half);
        }
        schedule_adaptive(
            my_half,
            self.partial_output,
            self.folder,
            self.reduce_function,
            self.block_sizes,
        )
    }

    fn schedule(mut self) -> F::Output {
        // TODO: automate this min everywhere ?
        // TODO: factorize a little bit
        // start by computing a little bit in order to get a first output
        let size = min(self.input.base_length(), self.current_block_size);

        if self.input.base_length() <= self.current_block_size {
            let (io, i) = self.folder.fold(self.partial_output, self.input, size);
            return self.folder.to_output(io, i);
        } else {
            SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
            let (new_partial_output, new_input) =
                self.folder.fold(self.partial_output, self.input, size);
            self.partial_output = new_partial_output;
            self.input = new_input;
            SEQUENCE.with(|s| *s.borrow_mut() = false);
            if self.input.base_length() == 0 {
                //TODO ASK is this a redundant check?
                // it's over
                return self.folder.to_output(self.partial_output, self.input);
            }
        }

        // loop while not stolen or something left to do
        loop {
            if self.is_stolen() && self.input.base_length() > self.min_block_size {
                return self.answer_steal();
            }
            self.current_block_size = min(self.current_block_size * 2, self.max_block_size);
            let size = min(self.input.base_length(), self.current_block_size);

            if self.input.base_length() <= self.current_block_size {
                SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
                let (io, i) = self.folder.fold(self.partial_output, self.input, size);
                SEQUENCE.with(|s| *s.borrow_mut() = false);
                return self.folder.to_output(io, i);
            }
            SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
            let result = self.folder.fold(self.partial_output, self.input, size);
            self.partial_output = result.0;
            self.input = result.1;
            SEQUENCE.with(|s| *s.borrow_mut() = false);
            if self.input.base_length() == 0 {
                return self.folder.to_output(self.partial_output, self.input);
            }
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
                    let option = sequential_task(1, 1, || receiver.recv());
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
