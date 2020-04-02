//! Skipable iterators.
use crate::dislocated::{Dislocated, DislocatedMut};
use crate::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use std::iter::Take;

pub struct Skipable<I, N, S> {
    iter: I,
    next_op: N,
    skip: S,
}

pub struct BorrowingSkipable<'a, I: Sync, N: Sync, S: Sync> {
    iter: either::Either<I, DislocatedMut<'a, I>>,
    count: usize,
    next_op: Dislocated<'a, N>,
    skip: Dislocated<'a, S>,
}

pub struct SeqSkipable<'a, I: Sync, N: Sync> {
    iter: DislocatedMut<'a, I>,
    next_op: Dislocated<'a, N>,
}

impl<'a, E, I, N> Iterator for SeqSkipable<'a, I, N>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    E: Send,
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        Some((self.next_op)(&mut self.iter))
    }
}

impl<I, N, S, E> ItemProducer for Skipable<I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type Item = E;
}

impl<'a, I, N, S, E> ItemProducer for BorrowingSkipable<'a, I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type Item = E;
}

impl<I, N, S, E> Powered for Skipable<I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type Power = Indexed;
}

impl<'e, I, N, S, E> ParBorrowed<'e> for Skipable<I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type Iter = BorrowingSkipable<'e, I, N, S>;
}

impl<'a, 'e, I, N, S, E> SeqBorrowed<'e> for BorrowingSkipable<'a, I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type Iter = Take<SeqSkipable<'e, I, N>>;
}

impl<'a, I, N, S, E> BorrowingParallelIterator for BorrowingSkipable<'a, I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    type ScheduleType = Adaptive;
    fn iterations_number(&self) -> usize {
        self.count
    }
    fn seq_borrow<'e>(&'e mut self, size: usize) -> <Self as SeqBorrowed<'e>>::Iter {
        self.count -= size;
        SeqSkipable {
            iter: match &mut self.iter {
                either::Either::Left(i) => DislocatedMut::new(i),
                either::Either::Right(i) => i.borrow_mut(),
            },
            next_op: self.next_op.clone(),
        }
        .take(size)
    }
    fn part_completed(&self) -> bool {
        self.iterations_number() >= self.micro_blocks_sizes().next().unwrap_or(0)
    }
}

impl<I, N, S, E> ParallelIterator for Skipable<I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    fn bound_iterations_number(&self, size: usize) -> usize {
        size
    }
    fn par_borrow<'e>(&'e mut self, size: usize) -> <Self as ParBorrowed<'e>>::Iter {
        BorrowingSkipable {
            iter: either::Either::Right(DislocatedMut::new(&mut self.iter)),
            count: size,
            next_op: Dislocated::new(&self.next_op),
            skip: Dislocated::new(&self.skip),
        }
    }
}

impl<'a, I, N, S, E> Divisible for BorrowingSkipable<'a, I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    fn should_be_divided(&self) -> bool {
        self.count > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let left_count = self.count / 2;
        let right_count = self.count - left_count;
        let (left_iter, right_iter) = match self.iter {
            either::Either::Left(mut i) => {
                let right_i = (self.skip)(&mut i, left_count);
                (either::Either::Left(i), either::Either::Left(right_i))
            }
            either::Either::Right(mut i) => {
                let mut right_i = (self.skip)(&mut i, left_count);
                std::mem::swap(&mut right_i, &mut i); // put it in owner's iter
                (either::Either::Left(right_i), either::Either::Right(i))
            }
        };
        let right = BorrowingSkipable {
            iter: right_iter,
            count: right_count,
            next_op: self.next_op.clone(),
            skip: self.skip.clone(),
        };
        self.count = left_count;
        self.iter = left_iter;
        (self, right)
    }
}

pub fn skip<I, N, S, E>(init: I, next_op: N, skip_op: S) -> Skipable<I, N, S>
where
    I: Sized + Send + Sync,
    N: Fn(&mut I) -> E + Sync,
    S: Fn(&mut I, usize) -> I + Sync,
    E: Send,
{
    Skipable {
        iter: init,
        next_op,
        skip: skip_op,
    }
}

/// Infinite iterator on successors.
/// You need to provide a fast way to `skip` elements
///
/// # Example
///
/// ```
/// use rayon_adaptive::prelude::*;
/// use rayon_adaptive::successors;
/// // let's fake a range just for testing
/// let s:u64 = successors(0u64,
///                    |&i| i+1,
///                    |&i, n| i+(n as u64)).take(10_000).sum();
/// assert_eq!(s, 5_000* 9_999)
/// ```
pub fn successors<T, F, S>(
    first: T,
    succ: F,
    skip_op: S,
) -> impl ParallelIterator<Power = Indexed, Item = T>
where
    T: Send + Sync,
    F: Fn(&T) -> T + Sync,
    S: Fn(&T, usize) -> T + Sync,
{
    skip(
        first,
        move |t| {
            let mut next_t = succ(t);
            std::mem::swap(&mut next_t, t);
            next_t
        },
        move |t, s| skip_op(t, s),
    )
}

/// Infinite Iterator on pseudo-random elements.
/// Good stuff : the randomness is fully deterministic and
/// independant from scheduling policies.
///
/// # Example
/// ```
/// use rayon_adaptive::prelude::*;
/// use rayon_adaptive::chacha_iter;
///
/// let seed = [
///        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3,
///        0, 0, 0, 0, 0, 0, 0,
/// ];
///
/// let v1:Vec<u64> = chacha_iter(seed.clone()).with_join_policy(10).take(100).collect();
/// let v2:Vec<u64> = chacha_iter(seed).with_join_policy(20).take(100).collect();
/// assert_eq!(v1, v2);
/// ```
pub fn chacha_iter<T>(seed: [u8; 32]) -> impl ParallelIterator<Item = T, Power = Indexed>
where
    T: Send + Sync,
    rand::distributions::Standard: rand::distributions::Distribution<T>,
{
    let mut rng = ChaChaRng::from_seed(seed);
    rng.gen::<T>();
    let pos_increment = rng.get_word_pos() as usize;
    rng.set_word_pos(0);
    skip(
        (0, rng),
        move |(c, rng)| {
            *c += pos_increment;
            rng.gen()
        },
        move |(c, rng), size| {
            let mut right_rng = rng.clone();
            let new_pos = *c + size * pos_increment;
            right_rng.set_word_pos(new_pos as u128);
            (new_pos, right_rng)
        },
    )
}
