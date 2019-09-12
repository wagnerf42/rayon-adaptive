// new traits
//use crate::Try;
use crate::iter::*;
use crate::scheduler::schedule_reduce;
use std::iter::successors;

pub struct NotIndexed();
pub struct Indexed();

pub trait Divisible: Sized {
    fn is_divisible(&self) -> bool;
    /// Divide Self into two parts.
    /// It's better if the two parts contain roughly an equivalent amount of work.
    /// For Indexed iterators we REQUIRE an object of size n to be cut into two objects of size
    /// floor(n/2), ceil(n/2).
    fn divide(self) -> (Self, Self);
}

pub trait ItemProducer: Sized {
    type Owner: for<'e> Borrowed<'e>
        + ItemProducer<Item = Self::Item, Owner = Self::Owner, Power = Self::Power>
        + ParallelIterator;
    type Item: Send + Sized;
    type Power;
}

pub trait Borrowed<'e>: ItemProducer {
    type ParIter: FiniteParallelIterator
        + Divisible
        + ItemProducer<Item = Self::Item, Owner = Self::Owner, Power = Self::Power>;
    type SeqIter: Iterator<Item = Self::Item>;
}

//TODO: there is a pb with rayon's "split"
// because it's infinite but we can't borrow on left.
// we also can't borrow sequentially.
// tree iterator CAN be borrowed sequentially be cannot be borrowed in //
pub trait ParallelIterator: Send + ItemProducer {
    /// This function is used by scheduler before asking a borrow.
    /// It asks you how much it would like and you reply how much you can give.
    fn bound_size(&self, size: usize) -> usize {
        size // this is the default for infinite iterators
    }
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter;

    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter;

    fn cloned<'a, T>(self) -> Cloned<Self>
    where
        T: 'a + Clone + Send + Sync, // TODO I need Sync here but rayon does not
        Self: ParallelIterator<Item = &'a T>,
    {
        Cloned { iterator: self }
    }

    fn map<F, R>(self, op: F) -> Map<Self, F>
    where
        R: Send,
        F: Fn(Self::Item) -> R + Send,
    {
        Map { op, iterator: self }
    }
    fn filter<P>(self, filter_op: P) -> Filter<Self, P>
    where
        P: Fn(&Self::Item) -> bool + Send + Sync,
    {
        Filter {
            iterator: self,
            filter_op,
        }
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
    fn iterator_fold<R, F>(self, fold_op: F) -> IteratorFold<Self, F>
    where
        R: Sized + Send,
        F: for<'e> Fn(<Self as Borrowed<'e>>::SeqIter) -> R + Sync,
    {
        IteratorFold {
            iterator: self,
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
}

pub trait FiniteParallelIterator: ParallelIterator {
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn bound_size(&self, size: usize) -> usize {
        std::cmp::min(self.len(), size) // this is the default for finite iterators
    }
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
    fn for_each<OP>(self, op: OP)
    where
        OP: Fn(Self::Item) + Sync + Send,
    {
        self.map(op).reduce(|| (), |(), ()| ())
    }
    // here goes methods which cannot be applied to infinite iterators like sum
}

pub trait IndexedParallelIterator: ParallelIterator<Power = Indexed> {
    fn take(self, n: usize) -> Take<Self> {
        Take { iterator: self, n }
    }
    //TODO: use IntoParallelIterator
    fn zip<B: IndexedParallelIterator>(self, zip_op: B) -> Zip<Self, B> {
        Zip { a: self, b: zip_op }
    }
}

impl<I> IndexedParallelIterator for I where I: ParallelIterator<Power = Indexed> {}
