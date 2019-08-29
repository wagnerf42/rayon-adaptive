// new traits
use crate::join::JoinPolicy;
use crate::map::Map;

pub trait Divisible: Sized {
    fn is_divisible(&self) -> bool;
    fn divide(self) -> (Self, Self);
}

pub trait ParallelIterator: Divisible + Send
where
    Self: for<'extraction> FinitePart<'extraction>,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as FinitePart<'extraction>>::Iter;
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
    //    fn with_rayon_policy(self) -> RayonPolicy<Self> {
    //        RayonPolicy {
    //            iterator: self,
    //            created_by: rayon::current_thread_index(),
    //            counter: (rayon::current_num_threads() as f64).log(2.0).ceil() as usize,
    //        }
    //    }
}

// This is niko's magic for I guess avoiding the lifetimes in the ParallelIterator trait itself
pub trait FinitePart<'extraction>: ItemProducer {
    type Iter: FiniteParallelIterator<Item = <Self as ItemProducer>::Item>;
}

pub trait ItemProducer {
    type Item: Send;
}

pub trait FiniteParallelIterator: ParallelIterator {
    type SequentialIterator: Iterator<Item = Self::Item>;
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn to_sequential(self) -> Self::SequentialIterator;
    // here goes methods which cannot be applied to infinite iterators like sum
}
