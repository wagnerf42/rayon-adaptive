//! Let factorize a huge amount of scheduling policies into one api.
use crate::depjoin;
use crate::folders::Folder;
use crate::traits::Divisible;
use crate::Policy;
use rayon;
use rayon::current_num_threads;
use std::cell::RefCell;
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};

// we use this boolean to prevent fine grain parallelism when coarse grain
// parallelism is still available in composed algorithms.
thread_local!(static SEQUENCE: RefCell<bool> = RefCell::new(false));

/// by default, min block size is log(n)
fn default_min_block_size(n: usize) -> usize {
    (n as f64).log(2.0).floor() as usize
}

/// by default, max block size is sqrt(n)
fn default_max_block_size(n: usize) -> usize {
    (n as f64).sqrt().ceil() as usize
}

/// compute a block size with the given function.
/// this allows us to ensure we enforce important bounds on sizes.
fn compute_size<F: Fn(usize) -> usize>(n: usize, sizing_function: F) -> usize {
    let p = current_num_threads();
    std::cmp::min(n / (2 * p), sizing_function(n))
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
    let block_size = match policy {
        Policy::Sequential => input.base_length(),
        Policy::DefaultPolicy => compute_size(input.base_length(), default_min_block_size),
        Policy::Join(block_size)
        | Policy::JoinContext(block_size)
        | Policy::DepJoin(block_size)
        | Policy::Adaptive(block_size, _) => block_size,
    };
    match policy {
        Policy::Sequential => schedule_sequential(input, folder),
        Policy::Join(_) => schedule_join(input, folder, reduce_function, block_size),
        Policy::JoinContext(_) => schedule_join_context(input, folder, reduce_function, block_size),
        Policy::DepJoin(_) => schedule_depjoin(input, folder, reduce_function, block_size),
        Policy::Adaptive(min_size, max_size) => SEQUENCE.with(|s| {
            if *s.borrow() {
                schedule_sequential(input, folder)
            } else {
                schedule_adaptive(
                    input,
                    folder.identity(),
                    folder,
                    reduce_function,
                    (|_| min_size, |_| max_size),
                )
            }
        }),
        Policy::DefaultPolicy => SEQUENCE.with(|s| {
            if *s.borrow() {
                schedule_sequential(input, folder)
            } else {
                schedule_adaptive(
                    input,
                    folder.identity(),
                    folder,
                    reduce_function,
                    (default_min_block_size, default_max_block_size),
                )
            }
        }),
    }
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
    sender: Sender<Option<F::Input>>,
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
        sender: Sender<Option<F::Input>>,
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
        self.sender.send(Some(his_half)).expect("sending failed");
        schedule_adaptive(
            my_half,
            self.partial_output,
            self.folder,
            self.reduce_function,
            self.block_sizes,
        )
    }

    fn cancel_stealing_task(&mut self) {
        self.sender.send(None).expect("canceling task failed");
    }

    fn schedule(mut self) -> F::Output {
        // TODO: automate this min everywhere ?
        // TODO: factorize a little bit
        // start by computing a little bit in order to get a first output
        let size = min(self.input.base_length(), self.current_block_size);

        if self.input.base_length() <= self.current_block_size {
            self.cancel_stealing_task(); // no need to keep people waiting for nothing
            let (io, i) = self.folder.fold(self.partial_output, self.input, size);
            return self.folder.to_output(io, i);
        } else {
            let (new_partial_output, new_input) =
                self.folder.fold(self.partial_output, self.input, size);
            self.partial_output = new_partial_output;
            self.input = new_input;
            if self.input.base_length() == 0 {
                // it's over
                self.cancel_stealing_task();
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
                self.cancel_stealing_task(); // no need to keep people waiting for nothing
                let (io, i) = self.folder.fold(self.partial_output, self.input, size);
                return self.folder.to_output(io, i);
            }
            SEQUENCE.with(|s| *s.borrow_mut() = true); // we force subtasks to work sequentially
            let result = self.folder.fold(self.partial_output, self.input, size);
            self.partial_output = result.0;
            self.input = result.1;
            SEQUENCE.with(|s| *s.borrow_mut() = false);
            if self.input.base_length() == 0 {
                // it's over
                self.cancel_stealing_task();
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
        let (sender, receiver) = channel();

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
                #[cfg(feature = "logs")]
                let input =
                    rayon::sequential_task(1, 1, || receiver.recv().expect("receiving failed"))?;
                #[cfg(not(feature = "logs"))]
                let input = receiver.recv().expect("receiving failed")?;
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

// pub fn fully_adaptive_schedule<I, WF, RETF>(input: I, work_function: &WF, retrieve_function: &RETF)
// where
//     I: Divisible,
//     WF: Fn(I, usize) -> I + Sync,
//     RETF: Fn(I, I, I) -> I + Sync,
// {
//     //so, what kind of communications do we have ?
//     // * main thread is stolen
//     //   - stolen input
//     //   -
//     // * main thread retrieves data
//     // * helper thread is stolen
//     unimplemented!()
// }
