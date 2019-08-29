// new traits
use crate::join::JoinPolicy;
use crate::local::DampenLocalDivision;
use crate::map::Map;

pub trait Divisible: Sized {
    fn is_divisible(&self) -> bool;
    fn divide(self) -> (Self, Self);
}

pub trait ParallelIterator: Send + Sized
where
    Self: for<'extraction> FinitePart<'extraction>,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::ParIter;

    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::SeqIter;

    fn map<F, R>(self, op: F) -> Map<Self, F>
    where
        R: Send,
        F: Fn(<Self as ItemProducer>::Item) -> R + Send + Clone,
    {
        Map { op, iterator: self }
    }
    fn with_join_policy(self, fallback: usize) -> JoinPolicy<Self> {
        JoinPolicy {
            iterator: self,
            fallback,
        }
    }
    fn with_rayon_policy(self) -> DampenLocalDivision<Self> {
        DampenLocalDivision {
            iterator: self,
            created_by: rayon::current_thread_index(),
            counter: (rayon::current_num_threads() as f64).log(2.0).ceil() as usize,
        }
    }
}

// This is niko's magic for I guess avoiding the lifetimes in the ParallelIterator trait itself
pub trait FinitePart<'extraction>: ItemProducer {
    type ParIter: FiniteParallelIterator<Item = Self::Item>;
    type SeqIter: Iterator<Item = Self::Item>;
}

pub trait ItemProducer {
    type Item: Send;
}

pub trait FiniteParallelIterator: ParallelIterator + Divisible {
    type Iter: Iterator<Item = Self::Item>;
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn to_sequential(self) -> Self::Iter;
    // here goes methods which cannot be applied to infinite iterators like sum
}
