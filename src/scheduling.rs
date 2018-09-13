//! Let factorize a huge amount of scheduling policies into one api.
use depjoin;
use rayon;
use std::cmp::min;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use traits::{Divisible, Mergeable};

/// All scheduling available scheduling policies.
pub enum Policy {
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

pub(crate) fn schedule<D, M, F, G>(
    input: D,
    work_function: &F,
    output_function: &G,
    policy: Policy,
) -> M
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    match policy {
        Policy::Join(block_size) => {
            schedule_join(input, work_function, output_function, block_size)
        }
        Policy::JoinContext(block_size) => {
            schedule_join_context(input, work_function, output_function, block_size)
        }
        Policy::DepJoin(block_size) => {
            schedule_depjoin(input, work_function, output_function, block_size)
        }
        Policy::Adaptive(block_size) => {
            schedule_adaptive(input, work_function, output_function, block_size)
        }
    }
}

fn schedule_join<D, M, F, G>(mut input: D, work: &F, output: &G, block_size: usize) -> M
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    let len = input.len();
    if len <= block_size {
        work(&mut input, len);
        output(input)
    } else {
        let (i1, i2) = input.split();
        let (r1, r2) = rayon::join(
            || schedule_join(i1, work, output, block_size),
            || schedule_join(i2, work, output, block_size),
        );
        r1.fuse(r2)
    }
}

fn schedule_join_context<D, M, F, G>(mut input: D, work: &F, output: &G, block_size: usize) -> M
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    let len = input.len();
    if len <= block_size {
        work(&mut input, len);
        output(input)
    } else {
        let (i1, mut i2) = input.split();
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context(i1, work, output, block_size),
            |c| {
                if c.migrated() {
                    schedule_join_context(i2, work, output, block_size)
                } else {
                    let len = i2.len();
                    work(&mut i2, len);
                    output(i2)
                }
            },
        );
        r1.fuse(r2)
    }
}

fn schedule_depjoin<D, M, F, G>(mut input: D, work: &F, output: &G, block_size: usize) -> M
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    let len = input.len();
    if len <= block_size {
        work(&mut input, len);
        output(input)
    } else {
        let (i1, i2) = input.split();
        depjoin(
            || schedule_depjoin(i1, work, output, block_size),
            || schedule_depjoin(i2, work, output, block_size),
            |r1, r2| r1.fuse(r2),
        )
    }
}

struct AdaptiveWorker<
    'a,
    'b,
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync + 'b,
    G: Fn(D) -> M + Sync + 'b,
> {
    input: D,
    initial_block_size: usize,
    current_block_size: usize,
    stolen: &'a AtomicBool,
    sender: Sender<Option<D>>,
    work_function: &'b F,
    output_function: &'b G,
    phantom: PhantomData<(M)>,
}

impl<'a, 'b, D, M, F, G> AdaptiveWorker<'a, 'b, D, M, F, G>
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    fn new(
        input: D,
        initial_block_size: usize,
        stolen: &'a AtomicBool,
        sender: Sender<Option<D>>,
        work_function: &'b F,
        output_function: &'b G,
    ) -> Self {
        AdaptiveWorker {
            input,
            initial_block_size,
            current_block_size: initial_block_size,
            stolen,
            sender,
            work_function,
            output_function,
            phantom: PhantomData,
        }
    }
    fn work(&mut self, limit: usize) {
        (self.work_function)(&mut self.input, limit)
    }
    fn is_stolen(&self) -> bool {
        self.stolen.load(Ordering::Relaxed)
    }
    fn answer_steal(self) -> M {
        let (my_half, his_half) = self.input.split();
        self.sender.send(Some(his_half)).expect("sending failed");
        return schedule_adaptive(
            my_half,
            self.work_function,
            self.output_function,
            self.initial_block_size,
        );
    }

    fn cancel_stealing_task(&mut self) {
        self.sender.send(None).expect("canceling task failed");
    }

    //TODO: we still need macro blocks
    fn schedule(mut self) -> M {
        // TODO: automate this min everywhere ?
        // TODO: factorize a little bit
        // start by computing a little bit in order to get a first output
        let size = min(self.input.len(), self.current_block_size);

        if self.input.len() <= self.current_block_size {
            self.cancel_stealing_task(); // no need to keep people waiting for nothing
            self.work(size);
            return (self.output_function)(self.input);
        } else {
            self.work(size);
            if self.input.len() == 0 {
                // it's over
                self.cancel_stealing_task();
                return (self.output_function)(self.input);
            }
        }

        // I have this really nice proof as to why I need phi but the margins
        // are too small to write it down here :-)
        let phi: f64 = (1.0 + 5.0f64.sqrt()) / 2.0;

        // loop while not stolen or something left to do
        loop {
            if self.is_stolen() && self.input.len() > self.initial_block_size {
                return self.answer_steal();
            }
            self.current_block_size = (self.current_block_size as f64 * phi) as usize;
            let size = min(self.input.len(), self.current_block_size);

            if self.input.len() <= self.current_block_size {
                self.cancel_stealing_task(); // no need to keep people waiting for nothing
                self.work(size);
                return (self.output_function)(self.input);
            }
            self.work(size);
            if self.input.len() == 0 {
                // it's over
                self.cancel_stealing_task();
                return (self.output_function)(self.input);
            }
        }
    }
}

fn schedule_adaptive<D, M, F, G>(
    mut input: D,
    work_function: &F,
    output_function: &G,
    initial_block_size: usize,
) -> M
where
    D: Divisible,
    M: Mergeable,
    F: Fn(&mut D, usize) + Sync,
    G: Fn(D) -> M + Sync,
{
    let size = input.len();
    if size <= initial_block_size {
        work_function(&mut input, size);
        output_function(input)
    } else {
        let stolen = &AtomicBool::new(false);
        let (sender, receiver) = channel();

        let worker = AdaptiveWorker::new(
            input,
            initial_block_size,
            stolen,
            sender,
            work_function,
            output_function,
        );

        //TODO depjoin instead of join
        let (o1, maybe_o2) = rayon::join(
            move || {
                let r = worker.schedule();
                r
            },
            move || {
                stolen.store(true, Ordering::Relaxed);
                let received =
                    rayon::sequential_task(1, 1, || receiver.recv().expect("receiving failed"));
                if received.is_none() {
                    return None;
                }
                let input = received.unwrap();
                assert!(input.len() > 0);
                return Some(schedule_adaptive(
                    input,
                    work_function,
                    output_function,
                    initial_block_size,
                ));
            },
        );

        let fusion_needed = maybe_o2.is_some();
        if fusion_needed {
            o1.fuse(maybe_o2.unwrap())
        } else {
            o1
        }
    }
}
