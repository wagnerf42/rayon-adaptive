// new traits

pub trait Divisible: Sized {
    fn is_divisible(&self) -> bool;
    fn divide(self) -> (Self, Self);
}

pub trait Extractible: Sized
where
    Self: for<'extraction> ExtractiblePart<'extraction>,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as ExtractiblePart<'extraction>>::BorrowedPart;
    //    fn map<R: Send, F: Fn(I) -> R>(self, op: F) -> Map<I, Self, F> {
    //        Map {
    //            op,
    //            iterator: self,
    //            phantom: PhantomData,
    //        }
    //    }
}

// This is niko's magic for I guess avoiding the lifetimes in the Extractible trait itself
pub trait ExtractiblePart<'extraction>: ExtractibleItem {
    type BorrowedPart: ParallelIterator<Item = <Self as ExtractibleItem>::Item>;
}

pub trait ExtractibleItem {
    type Item: Send;
}

pub trait ParallelIterator: Divisible + Send {
    type Item: Send;
    type SequentialIterator: Iterator<Item = Self::Item>;
    fn len(&self) -> usize; // TODO: this should not be for all iterators
    fn to_sequential(self) -> Self::SequentialIterator;
    //    fn with_join_policy(self, fallback: usize) -> JoinPolicy<Self> {
    //        JoinPolicy {
    //            iterator: self,
    //            fallback,
    //        }
    //    }
    //    fn with_rayon_policy(self) -> RayonPolicy<Self> {
    //        RayonPolicy {
    //            iterator: self,
    //            created_by: rayon::current_thread_index(),
    //            counter: (rayon::current_num_threads() as f64).log(2.0).ceil() as usize,
    //        }
    //    }
}
