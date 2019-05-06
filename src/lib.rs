//! This crate provides mechanisms for designing adaptive algorithms for rayon.
#![type_length_limit = "2097152"]
#![warn(clippy::all)]
#![deny(missing_docs)]
/// Divisibility traits and implementations
pub(crate) mod divisibility;
/// Adaptive iterators
pub mod iter;
/// Import all traits in prelude to enable adaptive iterators.
pub mod prelude;
/// Different available scheduling policies.
pub enum Policy {
    /// Use rayon's scheduling algorithm.
    Rayon,
    /// Split recursively until given size is reached.
    Join(usize),
}
/// All scheduling algorithms.
pub(crate) mod schedulers;
