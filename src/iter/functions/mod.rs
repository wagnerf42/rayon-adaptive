//! We define here all functions returning `ParallelIterator`s like
//! `repeat`, `repeat_with`, `successors`.

mod successors;
pub use successors::successors;
