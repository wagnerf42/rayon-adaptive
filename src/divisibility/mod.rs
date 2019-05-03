//! We define here all divisibility traits and implement them
//! for basic types.
mod divisible;
pub use divisible::{BlocksIterator, Divisible, DivisibleAtIndex, DivisibleIntoBlocks};

// implement traits for all basic types
mod option;
mod range;
mod slice;
