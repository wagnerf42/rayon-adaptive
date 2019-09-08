// new traits
use crate::even_levels::EvenLevels;
//use crate::iterator_fold::IteratorFold;
use crate::join::JoinPolicy;
use crate::local::DampenLocalDivision;
use crate::map::Map;
use crate::scheduler::schedule_reduce;
use crate::Try;
use std::iter::successors;

pub trait Divisible: Sized {
    fn is_divisible(&self) -> bool;
    fn divide(self) -> (Self, Self);
}

pub trait ItemProducer {
    type Item: Send + Sized;
}

//TODO: there is a pb with rayon's "split"
// because it's infinite but we can't borrow on left.
// we also can't borrow sequentially.
// tree iterator CAN be borrowed sequentially be cannot be borrowed in //
pub trait ParallelIterator: Send + Sized
where
    Self: for<'extraction> Borrowed<'extraction>,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as Borrowed<'extraction>>::ParIter;

    fn sequential_borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as Borrowed<'extraction>>::SeqIter; // TODO: should we have a special method for last extraction ?

    fn map<F, R>(self, op: F) -> Map<Self, F>
    where
        R: Send,
        F: Fn(Self::Item) -> R + Send,
    {
        Map { op, iterator: self }
    }
    fn even_levels(self) -> EvenLevels<Self> {
        EvenLevels {
            even: true,
            iterator: self,
        }
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
    fn macro_blocks_sizes() -> Box<dyn Iterator<Item = usize>> {
        // TODO: should we go for a generic iterator type instead ?
        Box::new(successors(Some(rayon::current_num_threads()), |s| {
            Some(s * 2)
        }))
    }
    //    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<Self, F>
    //    where
    //        R: Sized + Send,
    //        F: for<'e> Fn(<Self as Borrowed<'e>>::SeqIter) -> R + Sync,
    //    {
    //        IteratorFold {
    //            iterator: self,
    //            fold_op,
    //        }
    //    }
    fn try_reduce<T, OP, ID>(self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(T, T) -> Self::Item + Sync + Send,
        ID: Fn() -> T + Sync + Send,
        Self::Item: Try<Ok = T>,
    {
        // loop on macro blocks until none are left or size is too small
        // create tasks until we cannot divide anymore
        // end with adaptive part using the micro blocks sizes iterator
        unimplemented!()
    }
}

// This is niko's magic for I guess avoiding the lifetimes in the ParallelIterator trait itself
pub trait Borrowed<'extraction>: ItemProducer {
    type ParIter: FiniteParallelIterator<
            Item = Self::Item, //ParIter = Self::ParIter,//TODO: get this to work
        > + Divisible;
    type SeqIter: Iterator<Item = Self::Item>;
}

pub trait FiniteParallelIterator: ParallelIterator {
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        let upper_bound = (self.len() as f64).sqrt().ceil() as usize;
        Box::new(successors(Some(1), move |s| {
            Some(std::cmp::min(s * 2, upper_bound))
        }))
    }
    fn reduce<ID, OP>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let len = self.len();
        let i = self.borrow_on_left_for(len);
        schedule_reduce(i, &identity, &op)
    }
    // here goes methods which cannot be applied to infinite iterators like sum
}
