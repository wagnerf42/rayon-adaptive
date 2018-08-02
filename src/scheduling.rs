//! Let factorize a huge amount of scheduling policies into one api.
use depjoin;
use rayon;
use std::cmp::min;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};

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

/// All inputs should implement this trait.
pub trait Block: Sized {
    type Output: Output;
    /// Return block's length.
    fn len(&self) -> usize {
        1
    }
    /// Divide ourselves.
    fn split(self) -> (Self, Self);
    /// Compute some output for this block. Return what's left to do if any and result.
    fn compute(self, limit: usize) -> (Option<Self>, Self::Output);
}

/// All outputs should implement this trait.
pub trait Output: Sized {
    /// Merge two outputs into one.
    fn fuse(self, other: Self) -> Self;
    /// Length of ouput.
    fn len(&self) -> usize {
        1
    }
}

impl Output for () {
    fn fuse(self, _other: Self) -> Self {
        ()
    }
    fn len(&self) -> usize {
        0
    }
}

pub fn schedule<B, R>(input: B, policy: Policy) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    match policy {
        Policy::Join(block_size) => schedule_join(input, block_size),
        Policy::JoinContext(block_size) => schedule_join_context(input, block_size),
        Policy::DepJoin(block_size) => schedule_depjoin(input, block_size),
        Policy::Adaptive(block_size) => schedule_adaptive(input, block_size),
    }
}

fn schedule_join<B, R>(input: B, block_size: usize) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    let len = input.len();
    if len < block_size {
        input.compute(len).1
    } else {
        let (i1, i2) = input.split();
        let (r1, r2) = rayon::join(
            || schedule_join(i1, block_size),
            || schedule_join(i2, block_size),
        );
        r1.fuse(r2)
    }
}

fn schedule_depjoin<B, R>(input: B, block_size: usize) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    let len = input.len();
    if len < block_size {
        input.compute(len).1
    } else {
        let (i1, i2) = input.split();
        depjoin(
            || schedule_depjoin(i1, block_size),
            || schedule_depjoin(i2, block_size),
            |r1, r2| r1.fuse(r2),
        )
    }
}

fn schedule_join_context<B, R>(input: B, block_size: usize) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    let len = input.len();
    if len < block_size {
        input.compute(len).1
    } else {
        let (i1, i2) = input.split();
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context(i1, block_size),
            |c| {
                if c.migrated() {
                    schedule_join_context(i2, block_size)
                } else {
                    let len = i2.len();
                    i2.compute(len).1
                }
            },
        );
        r1.fuse(r2)
    }
}

struct AdaptiveWorker<'a, B> {
    initial_block_size: usize,
    current_block_size: usize,
    stolen: &'a AtomicBool,
    sender: Sender<Option<B>>,
}

impl<'a, B: Block<Output = R> + Send, R: Output + Send> AdaptiveWorker<'a, B> {
    fn new(initial_block_size: usize, stolen: &'a AtomicBool, sender: Sender<Option<B>>) -> Self {
        AdaptiveWorker {
            initial_block_size,
            current_block_size: initial_block_size,
            stolen,
            sender,
        }
    }
    fn is_stolen(&self) -> bool {
        self.stolen.load(Ordering::Relaxed)
    }
    fn answer_steal(&mut self, input: B) -> R {
        let (my_half, his_half) = input.split();
        self.sender.send(Some(his_half)).expect("sending failed");
        return schedule_adaptive(my_half, self.initial_block_size);
    }

    fn cancel_stealing_task(&mut self) {
        self.sender.send(None).expect("canceling task failed");
    }

    //TODO: we still need macro blocks
    fn schedule(mut self, mut input: B) -> R {
        // start by computing a little bit in order to get a first output
        let size = min(input.len(), self.current_block_size);

        if input.len() <= self.current_block_size {
            self.cancel_stealing_task(); // no need to keep people waiting for nothing
        }
        let (mut remaining_input, mut output) = input.compute(size);

        // I have this really nice proof as to why I need phi but the margins
        // are too small to write it down here :-)
        let phi: f64 = (1.0 + 5.0f64.sqrt()) / 2.0;

        // loop while not stolen or something left to do
        loop {
            if remaining_input.is_none() {
                return output;
            }
            input = remaining_input.unwrap();

            //TODO: we need to better check if the input is splittable
            if self.is_stolen() && input.len() > self.initial_block_size {
                let new_output = self.answer_steal(input);
                return output.fuse(new_output);
            }
            self.current_block_size = (self.current_block_size as f64 * phi) as usize;

            if input.len() <= self.current_block_size {
                self.cancel_stealing_task(); // no need to keep people waiting for nothing
            }

            let size = min(input.len(), self.current_block_size);
            let (remaining, new_output) = input.compute(size);
            remaining_input = remaining;
            output = output.fuse(new_output);
        }
    }
}

fn schedule_adaptive<B, R>(input: B, initial_block_size: usize) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    let size = input.len();
    if size <= initial_block_size {
        input.compute(size).1
    } else {
        let stolen = &AtomicBool::new(false);
        let (sender, receiver) = channel();

        let worker = AdaptiveWorker::new(initial_block_size, stolen, sender);

        //TODO depjoin instead of join
        let (o1, maybe_o2) = rayon::join(
            move || worker.schedule(input),
            move || {
                stolen.store(true, Ordering::Relaxed);
                let received =
                    rayon::sequential_task(1, 1, || receiver.recv().expect("receiving failed"));
                if received.is_none() {
                    return None;
                }
                let input = received.unwrap();
                assert!(input.len() > 0);
                return Some(schedule_adaptive(input, initial_block_size));
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
