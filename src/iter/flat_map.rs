//! Implementation of flatmap.
use crate::prelude::*;
use either::Either;
use std::iter;
use std::iter::repeat;
use std::marker::PhantomData;

/// OUT is outer iterator type.
/// IN is inner iterator type.
/// F is the conversion function.
/// P is the power of the flatmap which is the not indexed power of the outer iterator.
/// PIN is the power of the inner iterator. it has no purpose but the compiler
/// forces us to have it here.
/// invariant: WE SHOULD NEVER BE OF SIZE ONE ON OUTER ITERATOR
pub enum FlatMap<P, PIN, OUT, IN, F> {
    /// We still have some content on the outer level
    OuterIterator(OUT, F, PhantomData<P>),
    /// Only content left is at inner level
    InnerIterator(IN, PhantomData<PIN>),
}

impl<P, PIN, OUT, IN, F, INTO> FlatMap<P, PIN, OUT, IN, F>
where
    P: Power,
    PIN: Power,
    OUT: ParallelIterator<P>,
    F: Fn(OUT::Item) -> INTO + Clone,
    INTO: IntoParallelIterator<PIN, Iter = IN>,
    IN: ParallelIterator<PIN, Item = INTO::Item>,
{
    fn new(out: OUT, map_op: F) -> Self {
        let length = out.base_length();
        if length == Some(1) {
            let (mut outer_iterator_sequential, _) = out.iter(1);
            let final_outer_element = outer_iterator_sequential.next().unwrap();
            let inner_parallel_iterator = map_op(final_outer_element).into_par_iter();
            FlatMap::InnerIterator(inner_parallel_iterator, Default::default())
        } else {
            FlatMap::OuterIterator(out, map_op, Default::default())
        }
    }
}

impl<P, PIN, OUT, IN, F, INTO> Divisible<P::NotIndexed> for FlatMap<P, PIN, OUT, IN, F>
where
    P: Power,
    PIN: Power,
    OUT: ParallelIterator<P>,
    F: Fn(OUT::Item) -> INTO + Clone,
    INTO: IntoParallelIterator<PIN, Iter = IN>,
    IN: ParallelIterator<PIN, Item = INTO::Item>,
{
    fn base_length(&self) -> Option<usize> {
        match self {
            FlatMap::OuterIterator(i, _, _) => i.base_length(),
            FlatMap::InnerIterator(i, _) => i.base_length(),
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        match self {
            FlatMap::OuterIterator(i, f, _) => {
                let (left, right) = i.divide_at(index);
                (
                    FlatMap::OuterIterator(left, f.clone(), Default::default()),
                    FlatMap::new(right, f),
                )
            }
            FlatMap::InnerIterator(i, _) => {
                let (left, right) = i.divide_at(index);
                (
                    FlatMap::InnerIterator(left, Default::default()),
                    FlatMap::InnerIterator(right, Default::default()),
                )
            }
        }
    }
}

impl<P, PIN, OUT, IN, F, INTO> ParallelIterator<P::NotIndexed> for FlatMap<P, PIN, OUT, IN, F>
where
    P: Power,
    PIN: Power,
    OUT: ParallelIterator<P>,
    F: Fn(OUT::Item) -> INTO + Clone + Send,
    INTO: IntoParallelIterator<PIN, Iter = IN>,
    IN: ParallelIterator<PIN, Item = INTO::Item>,
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
    fn iter(self, size: usize) -> (Self::SequentialIterator, Self) {
        match self {
            FlatMap::OuterIterator(i, f, _) => {
                let (outer_sequential_iterator, remaining_outer_iterator) = i.iter(size);
                (
                    Either::Left(
                        // we need to zip with the repeated f in order to pass a function and not a
                        // closure to flat_map.
                        // however I don't get why I need to cast manually.
                        // if I don't it complains about receiving a fn item.
                        outer_sequential_iterator.zip(repeat(f.clone())).flat_map(
                            map_par_to_seq as fn((OUT::Item, F)) -> IN::SequentialIterator,
                        ),
                    ),
                    FlatMap::new(remaining_outer_iterator, f),
                )
            }
            FlatMap::InnerIterator(i, _) => {
                let (inner_sequential_iterator, remaining_inner_iterator) = i.iter(size);
                (
                    Either::Right(inner_sequential_iterator),
                    FlatMap::InnerIterator(remaining_inner_iterator, Default::default()),
                )
            }
        }
    }
}

/// Turn outer parallel iter into flatmaped inner sequential iterator.
fn map_par_to_seq<E, F, INTO, PIN>(
    t: (E, F),
) -> <INTO::Iter as ParallelIterator<PIN>>::SequentialIterator
where
    PIN: Power,
    F: Fn(E) -> INTO + Clone,
    INTO: IntoParallelIterator<PIN>,
{
    let (e, fun) = t;
    let par_iter = fun(e).into_par_iter();
    let size = par_iter
        .base_length()
        .expect("cannot flat_map into infinite iterators");
    //TODO: technically we could but we need a method to extract the full iterator
    //it would be bad performance wise though
    let (seq_iter, _) = par_iter.iter(size);
    seq_iter
}
