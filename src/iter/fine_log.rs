use crate::prelude::*;

pub struct FineLog<I> {
    pub(crate) base: I,
    pub(crate) tag: &'static str,
}

/// Sequential Logged Iterator.
#[cfg(feature = "logs")]
pub struct LoggedIterator<I> {
    base: I,
    tag: &'static str,
    size: usize,
}
#[cfg(not(feature = "logs"))]
pub struct LoggedIterator<I> {
    base: I,
}

impl<I: Iterator> Iterator for LoggedIterator<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.base.next()
    }
}

impl<I> Drop for LoggedIterator<I> {
    fn drop(&mut self) {
        #[cfg(feature = "logs")]
        rayon_logs::end_subgraph(self.tag, self.size)
    }
}

impl<I: ItemProducer> ItemProducer for FineLog<I> {
    type Item = I::Item;
}

impl<I: Powered> Powered for FineLog<I> {
    type Power = I::Power;
}

impl<'e, I: ParallelIterator> ParBorrowed<'e> for FineLog<I> {
    type Iter = FineLog<<I as ParBorrowed<'e>>::Iter>;
}

impl<'e, I: BorrowingParallelIterator> SeqBorrowed<'e> for FineLog<I> {
    type Iter = LoggedIterator<<I as SeqBorrowed<'e>>::Iter>;
}

impl<I: Divisible> Divisible for FineLog<I> {
    fn should_be_divided(&self) -> bool {
        self.base.should_be_divided()
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.base.divide();
        (
            FineLog {
                base: left,
                tag: self.tag,
            },
            FineLog {
                base: right,
                tag: self.tag,
            },
        )
    }
}

impl<I: ParallelIterator> ParallelIterator for FineLog<I> {
    fn bound_iterations_number(&self, size: usize) -> usize {
        self.base.bound_iterations_number(size)
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        FineLog {
            base: self.base.par_borrow(size),
            tag: self.tag,
        }
    }
}

impl<I: BorrowingParallelIterator> BorrowingParallelIterator for FineLog<I> {
    fn iterations_number(&self) -> usize {
        self.base.iterations_number()
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        let r;
        #[cfg(feature = "logs")]
        {
            rayon_logs::start_subgraph(self.tag);
            let size = std::cmp::min(self.base.iterations_number(), size);
            r = LoggedIterator {
                base: self.base.seq_borrow(size),
                tag: self.tag,
                size,
            };
        }
        #[cfg(not(feature = "logs"))]
        {
            r = LoggedIterator {
                base: self.base.seq_borrow(size),
            };
        }
        r
    }
}
