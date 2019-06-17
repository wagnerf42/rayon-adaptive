use crate::help_work::HelpWork;
use crate::iter::{Cut, Work};
use std::cmp::min;
use std::iter::empty;
use std::mem;

/// This is a marker type for specialization

pub struct BasicPower();
/// This is a marker type for specialization

pub struct BlockedPower();
/// This is a marker type for specialization

pub struct IndexedPower();

/// To constrain types a little bit all markers need to implement this.
pub trait Power: Send {
    /// Power we have if we lose indexing.
    type NotIndexed: Power;
} // TODO: why on earth is the compiler requesting send ?
impl Power for BasicPower {
    type NotIndexed = BasicPower;
}
impl Power for BlockedPower {
    type NotIndexed = BlockedPower;
}
impl Power for IndexedPower {
    type NotIndexed = BlockedPower;
}
pub trait BlockedPowerOrMore: Power {}
impl BlockedPowerOrMore for BlockedPower {}
impl BlockedPowerOrMore for IndexedPower {}

/// All parallelism provided by this library is an abstraction over divide and conquer.
/// Parallel iterators can be divided into smaller parallel iterators but not only them,
/// we can provide parallel algorithms on objects which are NOT iterators like slices.
///
/// The main trait enabling us this abstraction is the `Divisible` trait enabling us
/// to divide an object into independant pieces.
///
/// All parallel objects implement this trait : all parallel iterators, slices, ranges, options,...
///
/// Not that ALL divisible types can reach a size of 0 which means division will never fail.
///
/// It directly comes with parallel algorithms even if the `Divisible` type is not an iterator.
pub trait Divisible: Sized {
    /// In rayon-adaptive we distinguish between several type of divisibility.
    /// The most basic object can be split in two on demand but no control
    /// is possible on the split point.
    /// Think for example of a parallel iterator on a binary tree.
    /// You can split only into left and right subtrees.
    ///
    /// Then above that you can get some control about the load balancing by dividing at
    /// around a given index. Think for example of a parallel iterator on keys of a hash table.
    /// Even if you do not know how many keys are available in the table you can still split
    /// exactly where you want in the table.
    ///
    /// The last type of `Divisible` objects can be split with an exact control
    /// of the sizes of the obtained pieces (range or slice and their corresponding iterators).
    ///
    /// We specialize many algorithms to get higher performances according to how your types allow
    /// to be divided. For example collecting from an indexed iterator into a vector is very fast but collecting
    /// from a non-indexed (filtered for example) iterator can be much slower.
    /// In order to choose what is the best algorithm AT COMPILE TIME we need to remember what is
    /// the true Power of the type.
    type Power: Power;

    /// The `base_length` function is both very simple and sometimes quite subtle to implement.
    /// A `Divisible` type needs to provide us with a size corresponding to the amount of
    /// iterations it contains. This information is crucial for inner scheduling algorithms.
    ///
    /// The size returned is an Option because we allow types of potentially infinite sizes like
    /// `Repeat` or infinite ranges. This types should return None to signal an unbounded size.
    ///
    /// The rules about the sizes returned are the following :
    /// - performances are best if you know and say the amount of work (usually iterations) inside
    /// self
    /// - if you say 0 it means all computations are now completed
    /// - it does not need to be a decreasing function : for example when using a flat_map we start
    /// with sizes of the outer parallel iterator but once we start parallelising the inner
    /// iterator we switche also to its sizes. It's then perfectly possible to have an object of
    /// size 2 being divided into two objects of larger sizes.
    fn base_length(&self) -> Option<usize>;

    /// Cut the `Divisible` into two parts, if possible at given index.
    /// You must implement this function even for low-power types (just discard the index).
    /// All algorithms will guarantee you that they will never call with and index > base_length.
    /// The index can be 0 though.
    fn divide_at(self, index: usize) -> (Self, Self);

    /// Cut the `Divisible` into two parts.
    /// By default we cut at midpoint of base_length.
    fn divide(self) -> (Self, Self) {
        let mid = self
            .base_length()
            .expect("cannot divide by default with no size")
            / 2;
        self.divide_at(mid)
    }

    /// This is a convenience function to `divide_at` and put back
    /// the left part in self. Doing this we don't really need to
    /// move out of self ; a mutable borrow is enough.
    fn borrow_divide_at(&mut self, index: usize) -> Self {
        let moved_self = unsafe { (self as *mut Self).read() };
        let (left, right) = moved_self.divide_at(index);
        unsafe { (self as *mut Self).write(left) };
        right
    }

    /// This is a convenience function to `divide` and put back
    /// the left part in self. Doing this we don't really need to
    /// move out of self ; a mutable borrow is enough.
    fn borrow_divide(&mut self) -> Self {
        let moved_self = unsafe { (self as *mut Self).read() };
        let (left, right) = moved_self.divide();
        unsafe { (self as *mut Self).write(left) };
        right
    }

    /// Return a sequential iterator on blocks of Self of given sizes.
    fn blocks<S: Iterator<Item = usize>>(self, sizes: S) -> BlocksIterator<Self, S> {
        BlocksIterator {
            sizes,
            remaining: Some(self),
        }
    }

    /// Work on ourselves piece by piece until length reaches 0.
    /// This is a complex algorithm. See `examples/prefix.rs`.
    fn work<W: Fn(Self, usize) -> Self + Send + Clone>(self, work_op: W) -> Work<Self, W> {
        Work {
            remaining_input: Some(self),
            work_op,
        }
    }

    /// Get a parallel iterator on parts of self divided by the load balancing algorithm.
    /// See `example/cut.rs`.
    fn cut(self) -> Cut<Self> {
        Cut { input: self }
    }

    /// This is the most complex operation available in the library.
    /// One thread will work sequentially with an optimal algorithm while
    /// all the other become helper threads and do a different operation.
    ///
    /// In this version of this algorithm helper threads do not work on iterators
    /// but directly consume the `Divisible`.
    /// See `examples/adaptive_prefix.rs`.
    ///
    /// This function registers the helper threads operations.
    fn with_help_work<H>(self, help_op: H) -> HelpWork<Self, H>
    where
        H: Fn(Self, usize) -> Self,
    {
        HelpWork {
            input: self,
            help_op,
            sizes: Box::new(empty()),
        }
    }
}

/// Iterator on some `Divisible` input by blocks.
pub struct BlocksIterator<I: Divisible, S: Iterator<Item = usize>> {
    /// sizes of all the remaining blocks
    pub(crate) sizes: S,
    /// remaining input
    pub(crate) remaining: Option<I>,
}

impl<I: Divisible, S: Iterator<Item = usize>> Iterator for BlocksIterator<I, S> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_none() {
            // no input left
            return None;
        }

        let remaining_length = self.remaining.as_ref().and_then(I::base_length);
        if let Some(length) = remaining_length {
            if length == 0 {
                // no input left
                return None;
            }
        }

        let current_size = self.sizes.next();
        if let Some(size) = current_size {
            let remaining_input = self.remaining.take().unwrap();
            let checked_size = remaining_length.map(|r| min(r, size)).unwrap_or(size);
            let (left, right) = remaining_input.divide_at(checked_size);
            mem::replace(&mut self.remaining, Some(right));
            Some(left)
        } else {
            // no sizes left, return everything thats left to process
            self.remaining.take()
        }
    }
}
