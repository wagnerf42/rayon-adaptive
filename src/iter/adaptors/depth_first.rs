//! Depth first adaptor
use crate::prelude::*;
use std::iter::FlatMap;

/// Depth first scheduling adaptor
pub struct DepthFirst<I> {
    /// all tasks form a stack from oldest (and largest) task to newest (and smallest)
    pub tasks: Vec<(I, usize)>,
}

impl<I: ParallelIterator> Divisible for DepthFirst<I> {
    type Power = I::Power;

    fn base_length(&self) -> Option<usize> {
        if self.tasks.len() > 1 {
            Some(self.tasks.len())
        } else if let Some((_, depth)) = self.tasks.last() {
            Some(1 + *depth)
        } else {
            Some(0)
        }
    }

    fn divide_at(mut self, _index: usize) -> (Self, Self) {
        if let Some((mut left, mut depth)) = self.tasks.pop() {
            while depth != 0 {
                if left.base_length().expect("infinite not supported") <= 1 {
                    depth = 0;
                } else {
                    let (new_left, right) = left.divide();
                    left = new_left;
                    depth -= 1;
                    self.tasks.push((right, depth));
                }
            }

            (
                DepthFirst {
                    tasks: vec![(left, 0usize)],
                },
                self,
            )
        } else {
            (self, DepthFirst { tasks: Vec::new() })
        }
    }
}

impl<I: ParallelIterator> ParallelIterator for DepthFirst<I> {
    type Item = I::Item;

    type SequentialIterator = FlatMap<
        std::vec::IntoIter<(I, usize)>,
        I::SequentialIterator,
        fn((I, usize)) -> I::SequentialIterator,
    >;

    fn extract_iter(&mut self, _size: usize) -> Self::SequentialIterator {
        self.tasks
            .pop()
            .into_iter()
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|(i, _)| i.to_sequential())
    }

    fn to_sequential(self) -> Self::SequentialIterator {
        self.tasks
            .into_iter()
            .flat_map(|(i, _depth): (I, usize)| i.to_sequential())
    }
}
