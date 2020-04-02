use crate::dislocated::Dislocated;
use crate::prelude::*;

//TODO: we could be folding accross macro blocks but it's
//breaking rayon's api and a pain to write.
//is it worth it ???
pub struct Fold<I, ID, F> {
    pub(crate) base: I,
    pub(crate) identity: ID,
    pub(crate) fold_op: F,
}

impl<T, I, ID, F> ItemProducer for Fold<I, ID, F>
where
    T: Send,
    ID: Fn() -> T,
{
    type Item = T;
}

impl<I, ID, F> Powered for Fold<I, ID, F> {
    type Power = Standard;
}

impl<'e, T, I, ID, F> ParBorrowed<'e> for Fold<I, ID, F>
where
    T: Send,
    I: ParallelIterator,
    ID: Fn() -> T + Sync,
    F: Fn(T, I::Item) -> T + Sync,
{
    type Iter = BorrowingFold<'e, T, <I as ParBorrowed<'e>>::Iter, ID, F>;
}

impl<T, I, ID, F> ParallelIterator for Fold<I, ID, F>
where
    T: Send,
    I: ParallelIterator,
    ID: Fn() -> T + Sync,
    F: Fn(T, I::Item) -> T + Sync,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowingFold {
            current_value: Some((self.identity)()),
            base: self.base.par_borrow(size),
            identity: Dislocated::new(&self.identity),
            fold_op: Dislocated::new(&self.fold_op),
        }
    }
}

pub struct BorrowingFold<'a, T, I, ID: Sync, F: Sync> {
    current_value: Option<T>,
    base: I,
    identity: Dislocated<'a, ID>,
    fold_op: Dislocated<'a, F>,
}

impl<'a, T, I, ID, F> ItemProducer for BorrowingFold<'a, T, I, ID, F>
where
    T: Send,
    ID: Sync,
    F: Sync,
{
    type Item = T;
}

impl<'e, 'a, T, I, ID, F> SeqBorrowed<'e> for BorrowingFold<'a, T, I, ID, F>
where
    T: Send,
    ID: Sync,
    F: Sync,
{
    type Iter = std::option::IntoIter<T>;
}

impl<'a, T, I, ID, F> BorrowingParallelIterator for BorrowingFold<'a, T, I, ID, F>
where
    T: Send,
    I: BorrowingParallelIterator,
    ID: Fn() -> T + Sync,
    F: Fn(T, I::Item) -> T + Sync,
{
    type ScheduleType = I::ScheduleType;
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        let mut new_value = self.current_value.take().unwrap();
        for element in self.base.seq_borrow(size) {
            new_value = (self.fold_op)(new_value, element);
        }
        if self.base.iterations_number() == 0 {
            Some(new_value)
        } else {
            self.current_value = Some(new_value);
            None
        }
        .into_iter()
    }
    fn part_completed(&self) -> bool {
        self.base.part_completed()
    }
}

impl<'a, T, I, ID, F> Divisible for BorrowingFold<'a, T, I, ID, F>
where
    T: Send,
    I: BorrowingParallelIterator,
    ID: Fn() -> T + Sync,
    F: Fn(T, I::Item) -> T + Sync,
{
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            BorrowingFold {
                current_value: self.current_value,
                base: left,
                identity: self.identity.clone(),
                fold_op: self.fold_op.clone(),
            },
            BorrowingFold {
                current_value: Some((self.identity)()),
                base: right,
                identity: self.identity,
                fold_op: self.fold_op,
            },
        )
    }
}
