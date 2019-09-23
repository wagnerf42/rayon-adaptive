use crate::iter::*;
use crate::prelude::*;
use std::iter::successors;

//TODO: there is a pb with rayon's "split"
// because it's infinite but we can't borrow on left.
// we also can't borrow sequentially.
// tree iterator CAN be borrowed sequentially be cannot be borrowed in //
pub trait ParallelIterator: Send + ItemProducer {
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
