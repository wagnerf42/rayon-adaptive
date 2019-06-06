//! Implementation of flatmap.
use crate::prelude::*;
use either::Either;
use std::iter;
use std::iter::repeat;

/// OUT is outer iterator type.
/// IN is inner iterator type.
/// F is the conversion function.
/// P is the power of the flatmap which is the not indexed power of the outer iterator.
/// PIN is the power of the inner iterator. it has no purpose but the compiler
/// forces us to have it here.
/// invariant: WE SHOULD NEVER BE OF SIZE ONE ON OUTER ITERATOR
pub enum FlatMap<OUT, IN, F> {
    /// We still have some content on the outer level
    OuterIterator(OUT, F),
    /// Only content left is at inner level
    InnerIterator(IN),
}

impl<OUT, IN, F, INTO> FlatMap<OUT, IN, F>
where
    OUT: ParallelIterator,
    F: Fn(OUT::Item) -> INTO + Clone,
    INTO: IntoParallelIterator<Iter = IN>,
    IN: ParallelIterator<Item = INTO::Item>,
{
    fn new(out: OUT, map_op: F) -> Self {
        let length = out.base_length();
        if length == Some(1) {
            let mut outer_iterator_sequential = out.to_sequential();
            let final_outer_element = outer_iterator_sequential.next().unwrap();
            let inner_parallel_iterator = map_op(final_outer_element).into_par_iter();
            FlatMap::InnerIterator(inner_parallel_iterator)
        } else {
            FlatMap::OuterIterator(out, map_op)
        }
    }
}

impl<OUT, IN, F, INTO> Divisible for FlatMap<OUT, IN, F>
where
    OUT: ParallelIterator,
    F: Fn(OUT::Item) -> INTO + Clone,
    INTO: IntoParallelIterator<Iter = IN>,
    IN: ParallelIterator<Item = INTO::Item>,
{
    type Power = <<OUT as Divisible>::Power as Power>::NotIndexed;
    fn base_length(&self) -> Option<usize> {
        match self {
            FlatMap::OuterIterator(i, _) => i.base_length(),
            FlatMap::InnerIterator(i) => i.base_length(),
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        match self {
            FlatMap::OuterIterator(i, f) => {
                let (left, right) = i.divide_at(index);
                (
                    FlatMap::OuterIterator(left, f.clone()),
                    FlatMap::new(right, f),
                )
            }
            FlatMap::InnerIterator(i) => {
                let (left, right) = i.divide_at(index);
                (FlatMap::InnerIterator(left), FlatMap::InnerIterator(right))
            }
        }
    }
}

impl<OUT, IN, F, INTO> ParallelIterator for FlatMap<OUT, IN, F>
where
    OUT: ParallelIterator,
    F: Fn(OUT::Item) -> INTO + Clone + Send,
    INTO: IntoParallelIterator<Iter = IN>,
    IN: ParallelIterator<Item = INTO::Item>,
{
    type Item = IN::Item;
    type SequentialIterator = Either<
        iter::FlatMap<
            iter::Zip<OUT::SequentialIterator, iter::Repeat<F>>,
            IN::SequentialIterator,
            fn((OUT::Item, F)) -> IN::SequentialIterator,
        >,
        IN::SequentialIterator,
    >;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        match self {
            FlatMap::OuterIterator(i, f) => {
                let outer_sequential_iterator = i.extract_iter(size);
                Either::Left(
                    // we need to zip with the repeated f in order to pass a function and not a
                    // closure to flat_map.
                    // however I don't get why I need to cast manually.
                    // if I don't it complains about receiving a fn item.
                    outer_sequential_iterator
                        .zip(repeat(f.clone()))
                        .flat_map(map_par_to_seq as fn((OUT::Item, F)) -> IN::SequentialIterator),
                )
            }
            FlatMap::InnerIterator(i) => {
                let inner_sequential_iterator = i.extract_iter(size);
                Either::Right(inner_sequential_iterator)
            }
        }
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        match self {
            FlatMap::OuterIterator(i, f) => {
                let outer_sequential_iterator = i.to_sequential();
                Either::Left(
                    // we need to zip with the repeated f in order to pass a function and not a
                    // closure to flat_map.
                    // however I don't get why I need to cast manually.
                    // if I don't it complains about receiving a fn item.
                    outer_sequential_iterator
                        .zip(repeat(f.clone()))
                        .flat_map(map_par_to_seq as fn((OUT::Item, F)) -> IN::SequentialIterator),
                )
            }
            FlatMap::InnerIterator(i) => {
                let inner_sequential_iterator = i.to_sequential();
                Either::Right(inner_sequential_iterator)
            }
        }
    }
}

/// Turn outer parallel iter into flatmaped inner sequential iterator.
fn map_par_to_seq<E, F, INTO>(t: (E, F)) -> <INTO::Iter as ParallelIterator>::SequentialIterator
where
    F: Fn(E) -> INTO + Clone,
    INTO: IntoParallelIterator,
{
    let (e, fun) = t;
    let par_iter = fun(e).into_par_iter();
    par_iter.to_sequential()
}
