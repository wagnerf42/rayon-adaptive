//! We define here all divisibility traits and implement them
//! for basic types.
mod divisible;
pub(crate) use divisible::BlockedPowerOrMore;
pub use divisible::{BasicPower, BlockedPower, BlocksIterator, Divisible, IndexedPower, Power};

// implement traits for all basic types
mod option;
mod range;
mod slice;
