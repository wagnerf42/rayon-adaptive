use rayon_adaptive::prelude::*;
use rayon_adaptive::IndexedPower;

struct ParSuccessors<T, F, S> {
    next: Option<T>,
    succ: F,
    skip_op: S,
}

impl<T, F, S> Divisible for ParSuccessors<T, F, S>
where
    T: Clone,
    F: Fn(T) -> Option<T> + Clone,
    S: Fn(T, usize) -> Option<T> + Clone,
{
    type Power = IndexedPower;
    fn base_length(&self) -> Option<usize> {
        if self.next.is_none() {
            Some(0)
        } else {
            None
        }
    }
    fn divide_at(self, index: usize) -> (Self, Self) {
        let right_next = self.next.clone().and_then(|v| (self.skip_op)(v, index));
        (
            ParSuccessors {
                next: self.next,
                succ: self.succ.clone(),
                skip_op: self.skip_op.clone(),
            },
            ParSuccessors {
                next: right_next,
                succ: self.succ.clone(),
                skip_op: self.skip_op.clone(),
            },
        )
    }
}

struct Successors<T, F> {
    next: Option<T>,
    succ: F,
    right_next: *mut Option<T>, // sadly we cannot borrow it because the associated type only has one lifetime
    remaining_iterations: usize,
}

impl<T, F, S> ParallelIterator for ParSuccessors<T, F, S>
where
    T: Clone + Send,
    F: Fn(T) -> Option<T> + Clone + Send,
    S: Fn(T, usize) -> Option<T> + Clone + Send,
{
    type Item = T;
    type SequentialIterator = Successors<T, F>;
    fn extract_iter(&mut self, size: usize) -> Self::SequentialIterator {
        let next = self.next.take();
        Successors {
            next,
            succ: self.succ.clone(),
            right_next: &mut self.next as *mut Option<T>,
            remaining_iterations: size,
        }
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        unimplemented!()
        // Successors {
        //     next: self.next,
        //     succ: self.succ,
        //     // TODO: pb both for right_next and remaining_iterations
        // }
    }
}

impl<T, F> Iterator for Successors<T, F>
where
    F: Fn(T) -> Option<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

fn main() {}
