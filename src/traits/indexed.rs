use crate::base::rangefrom::RangeFrom;
use crate::iter::*;
use crate::prelude::*;

type Enumerate<I> = Zip<RangeFrom<usize>, I>;

pub trait IndexedParallelIterator: ParallelIterator {
    /// # Example
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// let s:u32 = (0u32..10).into_par_iter()
    ///                      .take(5)
    ///                      .sum();
    /// assert_eq!(s, 10);
    /// let s: u32 = (0u32..).into_par_iter().take(100).sum();
    /// assert_eq!(s, 4950);
    /// ```
    fn take(self, len: usize) -> Take<Self> {
        Take {
            iterator: self,
            n: len,
        }
    }
    /// zip
    /// # Example:
    /// ```
    /// use rayon_adaptive::prelude::*;
    /// // 1,2,3 times 0,1,2,.. is 2,6 which sums to 8
    /// assert_eq!((1u32..4).into_par_iter().zip(0u32..8).map(|(e1, e2)| e1*e2).sum::<u32>(), 8)
    /// ```
    fn zip<Z>(self, zip_op: Z) -> Zip<Self, Z::Iter>
    where
        Z: IntoParallelIterator,
        Z::Iter: IndexedParallelIterator,
    {
        Zip {
            a: self,
            b: zip_op.into_par_iter(),
        }
    }
    fn enumerate(self) -> Enumerate<Self> {
        (0usize..).into_par_iter().zip(self)
    }
}

impl<I> IndexedParallelIterator for I where I: ParallelIterator<Power = Indexed> {}
