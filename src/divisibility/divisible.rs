use crate::Policy;

/// This is the first level of the divisibility traits hierarchy.
/// All parallel objects must at least implement this trait.
/// Note that this abstraction is stronger than parallel iterators and
/// will allow parallel operations on non-iterator objects.
pub trait Divisible: Sized {
    /// Return our size. This corresponds to the number of operations to be issued.
    /// For example *i.filter(f)* should have as size the number of elements in i before
    /// filtering. At size 0 nothing is left to do. Any `Divisible` of infinite size
    /// (like unbounded ranges) should return `None`.
    fn base_length(&self) -> Option<usize>;
    /// Cut the `Divisible` into two parts.
    fn divide(self) -> (Self, Self);
    /// Return current scheduling `Policy`.
    fn policy(&self) -> Policy {
        Policy::Rayon
    }
    // fn blocks_sizes(&self) ?????
}
