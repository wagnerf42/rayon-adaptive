use crate::scheduler::schedule_reduce;
mod divisible;
mod indexed;
mod into_iterator;
mod into_parallel_ref;
// mod parallel_iterator;
mod types;
use std::iter::successors;

pub use divisible::Divisible;
pub use indexed::IndexedParallelIterator;
pub use into_iterator::IntoParallelIterator;
pub use into_parallel_ref::IntoParallelRefIterator;
// pub use parallel_iterator::ParallelIterator;
pub use types::{Indexed, ItemProducer, MinPower, ParBorrowed, Powered, SeqBorrowed, Standard};

use crate::iter::*;
pub trait ParallelIterator: Powered + Sized
where
    Self: for<'e> ParBorrowed<'e>,
{
    /// Takes the number of iterations requested by the user
    /// and return the number we can really process.
    fn bound_iterations_number(&self, size: usize) -> usize;
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter;

    fn completed(&self) -> bool {
        self.bound_iterations_number(std::usize::MAX) == 0
    }

    fn chain<C>(self, chain: C) -> Chain<Self, C::Iter>
    where
        C: IntoParallelIterator<Item = Self::Item>,
        <C::Iter as Powered>::Power: MinPower<Self::Power>,
    {
        Chain {
            a: self,
            b: chain.into_par_iter(),
        }
    }

    /// fold
    /// # Example
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let v = (0u32..10).into_par_iter()
    ///                      .fold(Vec::new, |mut v, e| {v.push(e);v})
    ///                      .reduce(Vec::new, |mut v1, mut v2| {v1.append(&mut v2);v1});
    /// assert_eq!(v, vec![0,1,2,3,4,5,6,7,8,9])
    /// ```
    fn fold<T, ID, F>(self, identity: ID, fold_op: F) -> Fold<Self, ID, F>
    where
        T: Send,
        ID: Fn() -> T + Sync,
        F: Fn(T, Self::Item) -> T + Sync,
    {
        Fold {
            base: self,
            identity,
            fold_op,
        }
    }
    //    //    fn try_reduce<T, OP, ID>(self, identity: ID, op: OP) -> Self::Item
    //    //    where
    //    //        OP: Fn(T, T) -> Self::Item + Sync + Send,
    //    //        ID: Fn() -> T + Sync + Send,
    //    //        Self::Item: Try<Ok = T>,
    //    //    {
    //    //        // loop on macro blocks until none are left or size is too small
    //    //        // create tasks until we cannot divide anymore
    //    //        // end with adaptive part using the micro blocks sizes iterator
    //    //        unimplemented!()
    //    //    }

    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<Self, F>
    where
        R: Sized + Send,
        F: Fn(<<Self as ParBorrowed>::Iter as SeqBorrowed>::Iter) -> R + Sync,
    {
        IteratorFold {
            base: self,
            fold_op,
        }
    }

    fn with_join_policy(self, fallback: usize) -> JoinPolicy<Self> {
        JoinPolicy {
            base: self,
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

    /// filter.
    /// # Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0u32..10).into_par_iter().filter(|&e| e%2==0).sum::<u32>(), 20)
    /// ```
    fn filter<P>(self, filter_op: P) -> Filter<Self, P>
    where
        P: Fn(&Self::Item) -> bool + Sync,
    {
        Filter {
            iterator: self,
            filter_op,
        }
    }

    fn even_levels(self) -> EvenLevels<Self> {
        EvenLevels {
            even: true,
            base: self,
        }
    }

    fn map<F, R>(self, op: F) -> Map<Self, F>
    where
        R: Send,
        F: Fn(Self::Item) -> R + Send,
    {
        Map { op, base: self }
    }

    fn fine_log(self, tag: &'static str) -> FineLog<Self> {
        FineLog { base: self, tag }
    }

    fn cloned<'a, T>(self) -> Cloned<Self>
    where
        T: 'a + Clone + Send + Sync, // TODO I need Sync here but rayon does not
        Self: ParallelIterator<Item = &'a T>,
    {
        Cloned { base: self }
    }

    fn reduce<ID, OP>(mut self, identity: ID, op: OP) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        let mut reduced_value = identity();
        while !self.completed() {
            // TODO: we should use macro_blocks_sizes here
            let size = self.bound_iterations_number(std::usize::MAX);
            let block = self.par_borrow(size);
            reduced_value = block.block_reduce(&identity, &op, reduced_value);
        }
        reduced_value
    }

    /// Sums all content of the iterator.
    /// Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// assert_eq!((0u32..3).into_par_iter().sum::<u32>(), 3);
    /// ```
    fn sum<S>(self) -> S
    where
        S: Send + core::iter::Sum<S> + core::iter::Sum<Self::Item>,
    {
        //TODO: we are stuck with that until iterator_fold kicks in
        self.map(|e| std::iter::once(e).sum::<S>()).reduce(
            || std::iter::empty::<S>().sum::<S>(),
            |a, b| std::iter::once(a).chain(std::iter::once(b)).sum::<S>(),
        )
    }

    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).reduce(|| (), |(), ()| ())
    }

    /// # Example
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let s:u32 = (0u32..10).into_par_iter()
    ///                      .take(5)
    ///                      .sum();
    /// assert_eq!(s, 10);
    /// let s: u32 = (0u32..).into_par_iter().take(100).sum();
    /// assert_eq!(s, 4950);
    /// ```
    fn take(self, len: usize) -> Take<Self> {
        Take {
            iterator: self,
            n: len,
        }
    }
}

pub trait BorrowingParallelIterator: Divisible + ItemProducer + Send
where
    Self: for<'e> SeqBorrowed<'e>,
{
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter;
    /// Return the number of iterations we still need to do.
    fn iterations_number(&self) -> usize;
    /// Return if nothing is left to do.
    fn completed(&self) -> bool {
        self.iterations_number() == 0
    }
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        Box::new(std::iter::successors(Some(1), |i| Some(2 * i)))
    }
    /// Reduce on one block.
    fn block_reduce<ID, OP>(self, identity: ID, op: OP, init: Self::Item) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        schedule_reduce(self, &identity, &op, init)
    }
}
