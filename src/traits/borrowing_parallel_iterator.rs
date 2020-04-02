use crate::prelude::*;
use crate::scheduler::Schedulable;

pub trait BorrowingParallelIterator: Divisible + ItemProducer + Send
where
    Self: for<'e> SeqBorrowed<'e>,
{
    type ScheduleType: Schedulable;
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter;
    /// Return the number of iterations we still need to do.
    fn iterations_number(&self) -> usize;
    /// This method will be used to stop division prematurely if needed. For performance reasons,
    /// the parallel iterator may want to proceed sequentially with the remaining work, so while
    /// iterations_number is non-zero, one may still want to prevent division.
    /// If this returns false, the stealer will not be given anything in that run and the
    /// sequential iterator will be borrowed and folded.
    fn part_completed(&self) -> bool;
    fn micro_blocks_sizes(&self) -> Box<dyn Iterator<Item = usize>> {
        Box::new(std::iter::successors(Some(1), |i| Some(2 * i)))
    }
    fn next(&mut self) -> Option<Self::Item> {
        self.seq_borrow(1).next()
    }
    /// Reduce on one block.
    fn block_reduce<ID, OP>(self, identity: ID, op: OP, init: Self::Item) -> Self::Item
    where
        OP: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
        ID: Fn() -> Self::Item + Sync,
    {
        <Self::ScheduleType>::schedule_reduce(self, &identity, &op, init)
    }
}
