use crate::prelude::*;

/// this just does one block for now
pub(crate) fn schedule_reduce<I, ID, OP>(mut iterator: I, identity: &ID, op: &OP) -> I::Item
where
    I: FiniteParallelIterator + Divisible,
    OP: Fn(I::Item, I::Item) -> I::Item + Sync,
    ID: Fn() -> I::Item + Sync,
{
    // for now just a non adaptive version
    if iterator.is_divisible() {
        let (left, right) = iterator.divide();
        let (left_answer, right_answer) = rayon::join(
            || schedule_reduce(left, identity, op),
            || schedule_reduce(right, identity, op),
        );
        op(left_answer, right_answer)
    } else {
        let len = iterator.len();
        iterator
            .sequential_borrow_on_left_for(len)
            .fold(identity(), op)
    }
}
