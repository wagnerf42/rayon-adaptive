//! Let factorize a huge amount of scheduling policies into one api.
use rayon;

/// All scheduling available scheduling policies.
pub enum Policy {
    Join(u64),
}

/// All inputs should implement this trait.
pub trait Block: Sized {
    type Output: Output;
    /// Return block's length.
    fn len(&self) -> usize;
    /// Divide ourselves.
    fn split(self, mid: usize) -> (Self, Self);
    /// Compute output for this block.
    fn compute(self) -> Self::Output;
}

/// All outputs should implement this trait.
pub trait Output: Sized {
    /// Merge two outputs into one.
    fn fuse(self, other: Self) -> Self;
}

pub fn schedule<B, R>(input: B, policy: Policy) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    match policy {
        Policy::Join(block_size) => schedule_join(input, block_size),
    }
}

pub fn schedule_join<B, R>(input: B, block_size: u64) -> R
where
    B: Block<Output = R> + Send,
    R: Output + Send,
{
    if input.len() < block_size as usize {
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
