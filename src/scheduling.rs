//! Let factorize a huge amount of scheduling policies into one api.

/// All scheduling available scheduling policies.
pub enum Policy {
    Join(u64),
}

pub fn schedule<B, R, S, F, O>(input: B, split_op: S, fuse_op: F, op: O, policy: Policy) -> R
where
    S: Fn(B, usize) -> (B, B) + Sync,
    F: Fn(R, R) -> R + Sync,
    O: Fn(B) -> R + Sync,
{
    unimplemented!()
}
