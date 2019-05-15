//! This crate provides mechanisms for designing adaptive algorithms for rayon.
#![type_length_limit = "2097152"]
#![warn(clippy::all)]
#![deny(missing_docs)]

#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

/// Divisibility traits and implementations
pub(crate) mod divisibility;
pub use divisibility::{BasicPower, BlockedPower, IndexedPower};
/// Adaptive iterators
pub mod iter;
/// Import all traits in prelude to enable adaptive iterators.
pub mod prelude;
/// Different available scheduling policies.
#[derive(Debug, Clone, Copy)]
pub enum Policy {
    /// Use rayon's scheduling algorithm.
    Rayon(usize),
    /// Split recursively until given size is reached.
    Join(usize),
    /// Split adaptively according to steal requests.
    /// Local iterator sizes are between given sizes.
    Adaptive(usize, usize),
    /// Just run sequentially
    Sequential,
}
/// All scheduling algorithms.
pub(crate) mod schedulers;

pub(crate) mod smallchannel;
pub(crate) mod utils;
