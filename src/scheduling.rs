//! Let factorize a huge amount of scheduling policies into one api.
use depjoin;
use rayon;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;

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
    fn split(self, mid: usize) -> (Self, Self);
    /// Compute output for this block.
    fn compute(self) -> Self::Output;
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
    if input.len() < block_size {
        input.compute()
    } else {
        let midpoint = input.len() / 2;
        let (i1, i2) = input.split(midpoint);
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
    if input.len() < block_size {
        input.compute()
    } else {
        let midpoint = input.len() / 2;
        let (i1, i2) = input.split(midpoint);
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
    if input.len() < block_size {
        input.compute()
    } else {
        let midpoint = input.len() / 2;
        let (i1, i2) = input.split(midpoint);
        let (r1, r2) = rayon::join_context(
            |_| schedule_join_context(i1, block_size),
            |c| {
                if c.migrated() {
                    schedule_join_context(i2, block_size)
                } else {
                    i2.compute()
                }
            },
        );
        r1.fuse(r2)
    }
}

fn schedule_adaptive<B, R>(
    input: B,
    done: &mut Vec<R>,
    initial_block_size: usize,
    growth_factor: f64,
) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    let stolen = &AtomicBool::new(false);
    let (sender, receiver) = channel();

    //TODO depjoin instead of join
    let (o1, maybe_o2) = rayon::join(
        move || {
            let mut input = input;
            let mut block_size = initial_block_size;

            while input.len() > 0 {
                if input.len() > initial_block_size && stolen.load(Ordering::Relaxed) {
                    let mid = input.len() / 2;
                    let (my_half, his_half) = input.split(mid);
                    sender.send(Some(his_half)).expect("sending failed");
                    return schedule_adaptive(my_half, done, initial_block_size, growth_factor);
                }
                if block_size > input.len() {
                    block_size = input.len();
                }
                let (next_block, remaining_part) = input.split(block_size);
                let mut output = next_block.compute();
                loop {
                    // TODO: check for steal requests between each fusion ?
                    if !done.last()
                        .map(|last_output| output.len() >= last_output.len())
                        .unwrap_or(false)
                    {
                        break;
                    }
                    let last_output = done.pop().unwrap();
                    output = last_output.fuse(output);
                }
                done.push(output);
                input = remaining_part;
                block_size = (block_size as f64 * growth_factor) as usize;
            }
            let mut output = done.pop().unwrap();
            loop {
                if done.is_empty() {
                    sender.send(None).expect("sending none failed");
                    return output;
                }
                let last_output = done.pop().unwrap();
                output = last_output.fuse(output);
            }
        },
        move || {
            stolen.store(true, Ordering::Relaxed);
            let received = receiver.recv().expect("receiving failed");
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
