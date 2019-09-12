// map
use crate::dislocated::Dislocated;
use crate::prelude::*;

pub struct Map<I, F> {
    pub(crate) op: F,
    pub(crate) iterator: I,
}

impl<R, I, F> ItemProducer for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    type Owner = Map<I::Owner, F>;
    type Item = R;
}

// il comprend pas que  l'owner du ParIter est bien Self
// qui est l'owner du ParIter ?
// BMAP<I::Owner::ParIter>::Owner
// I est un iterateur parallele mais on ne sait pas de quel niveau
// supposons dans un premier temps qu'il est un owner
// on a donc I::Owner est I
// BMAP<I::Owner::ParIter> est donc BMAP<I::ParIter>
// l'owner de BMAP<x> est par def MAP<x::Owner>
// donc l'owner de BMAP<I::Owner::ParIter>
// est MAP<I::ParIter::Owner>
// l'owner d'un ParIter etant contraint a etre l'owner de son pere on a:
// MAP<I::ParIter::Owner> est MAP<I::Owner>
//
// supposons maintenant que I est deja emprunte
// BMAP<I::Owner::ParIter>::Owner
// est MAP<I::Owner::ParIter::Owner>
// donc MAP<I::Owner::Owner>
// donc MAP<I::Owner> d'apres la ligne 18 du prelude ?
impl<'e, R, I, F> Borrowed<'e> for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    type ParIter = BorrowingMap<'e, <I::Owner as Borrowed<'e>>::ParIter, F>;
    type SeqIter = SeqBorrowingMap<'e, <I::Owner as Borrowed<'e>>::SeqIter, F>;
}

impl<R, I, F> ParallelIterator for Map<I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self as Borrowed<'e>>::ParIter {
        BorrowingMap {
            iterator: self.iterator.borrow_on_left_for(size),
            op: Dislocated::new(&self.op),
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self as Borrowed<'e>>::SeqIter {
        SeqBorrowingMap {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            op: Dislocated::new(&self.op),
        }
    }
}

impl<R, I, F> FiniteParallelIterator for Map<I, F>
where
    I: FiniteParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

pub struct BorrowingMap<'e, I, F: Sync> {
    op: Dislocated<'e, F>,
    iterator: I,
}

impl<'e, I, F> Divisible for BorrowingMap<'e, I, F>
where
    I: Divisible,
    F: Sync,
{
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            BorrowingMap {
                op: self.op,
                iterator: left,
            },
            BorrowingMap {
                op: self.op,
                iterator: right,
            },
        )
    }
}

impl<'a, R, I, F> ItemProducer for BorrowingMap<'a, I, F>
where
    R: Send,
    I: ParallelIterator,
    F: Fn(I::Item) -> R + Send + Sync,
{
    type Owner = Map<I::Owner, F>;
    type Item = R;
}

impl<'a, R, I, F> ParallelIterator for BorrowingMap<'a, I, F>
where
    I: ParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Sync + Send,
{
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        BorrowingMap {
            iterator: self.iterator.borrow_on_left_for(size),
            op: self.op,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        SeqBorrowingMap {
            iterator: self.iterator.sequential_borrow_on_left_for(size),
            op: self.op,
        }
    }
}

impl<'e, R, I, F> FiniteParallelIterator for BorrowingMap<'e, I, F>
where
    I: FiniteParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

pub struct SeqBorrowingMap<'e, I, F: Sync> {
    iterator: I,
    op: Dislocated<'e, F>,
}

impl<'e, R, I, F> Iterator for SeqBorrowingMap<'e, I, F>
where
    I: Iterator,
    F: Fn(I::Item) -> R + Sync,
{
    type Item = R;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next().map(|e| (*self.op)(e))
    }
}

impl<R, I, F> IndexedParallelIterator for Map<I, F>
where
    I: IndexedParallelIterator,
    R: Send,
    F: Fn(I::Item) -> R + Send + Sync,
{
}
