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
    /// on the fly. Manage a stack of outputs and fuse them as a depth first execution
    /// (we try to fuse equal sized outputs).
    /// We need an initial block size and a block growing factor.
    Adaptive(usize, f64),
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
        Policy::Adaptive(block_size, growth_factor) => {
            schedule_adaptive(input, &mut Vec::new(), block_size, growth_factor)
        }
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

struct AdaptiveWorker<'a, 'b, B, R: 'b> {
    done: &'b mut Vec<(R, usize)>,
    initial_block_size: usize,
    current_block_size: usize,
    growth_factor: f64,
    stolen: &'a AtomicBool,
    sender: Sender<Option<B>>,
}

impl<'a, O: 'a + Output> Block for &'a mut [(O, usize)] {
    type Output = ();
    fn len(&self) -> usize {
        //TODO: this is just plain wrong
        self.last().map(|o| o.1).unwrap_or(0)
    }
    fn split(self) -> (Self, Self) {
        unimplemented!()
    }
    fn compute(self, limit: usize) -> (Option<Self>, ()) {
        unimplemented!()
    }
}

impl<'a, 'b, B: Block<Output = R> + Send, R: Output + Send> AdaptiveWorker<'a, 'b, B, R> {
    fn new(
        done: &'b mut Vec<(R, usize)>,
        initial_block_size: usize,
        growth_factor: f64,
        stolen: &'a AtomicBool,
        sender: Sender<Option<B>>,
    ) -> Self {
        AdaptiveWorker {
            done,
            initial_block_size,
            current_block_size: initial_block_size,
            growth_factor,
            stolen,
            sender,
        }
    }
    fn fusion_needed(&self) -> bool {
        self.done.last().map(|e| e.1 > 200_000).unwrap_or(false)
    }
    fn is_stolen(&self) -> bool {
        self.stolen.load(Ordering::Relaxed)
    }
    fn fuse_all_outputs(&mut self) -> R {
        //TODO: use schedule :-)
        let (mut last_output, _) = self.done.pop().unwrap();
        while let Some((next_to_last_output, _)) = self.done.pop() {
            last_output = next_to_last_output.fuse(last_output);
        }
        last_output
    }

    fn answer_steal(&mut self, input: B) -> R {
        let (my_half, his_half) = input.split();
        self.sender.send(Some(his_half)).expect("sending failed");
        return schedule_adaptive(
            my_half,
            self.done,
            self.initial_block_size,
            self.growth_factor,
        );
    }

    fn cancel_stealing_task(&mut self) {
        self.sender.send(None).expect("canceling task failed");
    }

    fn schedule(mut self, input: B) -> R {
        let mut input = input;
        while !self.fusion_needed() {
            let size = min(input.len(), self.current_block_size);
            let (remaining_part, output) = input.compute(size);
            self.current_block_size =
                (self.current_block_size as f64 * self.growth_factor) as usize;
            let previous_size = self.done.last().map(|o| o.1).unwrap_or(0);
            let new_size = previous_size + output.len();
            self.done.push((output, new_size));
            if remaining_part.is_none() {
                self.cancel_stealing_task();
                return self.fuse_all_outputs();
            }
            input = remaining_part.unwrap();

            //TODO: we need to better check if the input is splittable
            if self.is_stolen() && input.len() > self.initial_block_size {
                return self.answer_steal(input);
            }
        }
        self.cancel_stealing_task();
        self.fuse_all_outputs();
        return schedule_adaptive(
            input,
            self.done,
            self.initial_block_size,
            self.growth_factor,
        );
    }
}

fn schedule_adaptive<B, R>(
    input: B,
    done: &mut Vec<(R, usize)>,
    initial_block_size: usize,
    growth_factor: f64,
) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    //TODO: if output is not splittable do it sequentially
    let stolen = &AtomicBool::new(false);
    let (sender, receiver) = channel();

    let worker = AdaptiveWorker::new(done, initial_block_size, growth_factor, stolen, sender);

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
            return Some(schedule_adaptive(
                input,
                &mut Vec::new(),
                initial_block_size,
                growth_factor,
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
