use crate::prelude::*;

pub struct FineLog<I> {
    pub(crate) iterator: I,
    pub(crate) tag: &'static str,
}

/// Sequential Logged Iterator.
#[cfg(feature = "logs")]
pub struct LoggedIterator<I> {
    iterator: I,
    tag: &'static str,
    size: usize,
}
#[cfg(not(feature = "logs"))]
pub struct LoggedIterator<I> {
    iterator: I,
}

impl<I: Iterator> Iterator for LoggedIterator<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.iterator.next()
    }
}

impl<I> Drop for LoggedIterator<I> {
    fn drop(&mut self) {
        #[cfg(feature = "logs")]
        rayon_logs::end_subgraph(self.tag, self.size)
    }
}

impl<I: ParallelIterator> ItemProducer for FineLog<I> {
    type Item = I::Item;
    type Owner = FineLog<I::Owner>;
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> Borrowed<'e> for FineLog<I> {
    type ParIter = FineLog<<I::Owner as Borrowed<'e>>::ParIter>;
    type SeqIter = LoggedIterator<<I::Owner as Borrowed<'e>>::SeqIter>;
}

impl<I: Divisible> Divisible for FineLog<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            FineLog {
                iterator: left,
                tag: self.tag,
            },
            FineLog {
                iterator: right,
                tag: self.tag,
            },
        )
    }
}

impl<I: ParallelIterator> ParallelIterator for FineLog<I> {
    fn borrow_on_left_for<'e>(&'e mut self, size: usize) -> <Self::Owner as Borrowed<'e>>::ParIter {
        FineLog {
            iterator: self.iterator.borrow_on_left_for(size),
            tag: self.tag,
        }
    }
    fn sequential_borrow_on_left_for<'e>(
        &'e mut self,
        size: usize,
    ) -> <Self::Owner as Borrowed<'e>>::SeqIter {
        let r;
        #[cfg(feature = "logs")]
        {
            rayon_logs::start_subgraph(self.tag);
            let size = self.iterator.bound_size(size);
            r = LoggedIterator {
                iterator: self.iterator.sequential_borrow_on_left_for(size),
                tag: self.tag,
                size,
            };
        }
        #[cfg(not(feature = "logs"))]
        {
            r = LoggedIterator {
                iterator: self.iterator.sequential_borrow_on_left_for(size),
            };
        }
        r
    }
}

impl<I: FiniteParallelIterator> FiniteParallelIterator for FineLog<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}
