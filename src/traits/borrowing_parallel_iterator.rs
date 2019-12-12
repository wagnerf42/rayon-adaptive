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
        <Self::ScheduleType>::schedule_reduce(self, &identity, &op, init)
    }
}
