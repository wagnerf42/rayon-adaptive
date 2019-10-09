//! We re-export here all iterator adaptors.
mod chain;
mod cloned;
mod even_levels;
mod filter;
mod fine_log;
mod iterator_fold;
mod join;
mod local;
mod map;
// mod take;
// mod flatmap;
mod fold;
mod zip;
//
pub use chain::Chain;
pub use cloned::Cloned;
pub use even_levels::EvenLevels;
pub use filter::Filter;
pub use fine_log::FineLog;
pub use iterator_fold::IteratorFold;
pub use join::JoinPolicy;
pub use local::DampenLocalDivision;
pub use map::Map;
// pub use take::Take;
// pub use flatmap::FlatMap;
pub use fold::Fold;
pub use zip::Zip;
