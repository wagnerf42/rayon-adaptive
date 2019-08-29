struct JoinPolicy<I> {
    iterator: I,
    fallback: usize,
}

impl<I: ParallelIterator> Divisible for JoinPolicy<I> {
    fn is_divisible(&self) -> bool {
        // TODO: check if divisible ??
        self.iterator.is_divisible() && self.iterator.len() > self.fallback
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        (
            JoinPolicy {
                iterator: left,
                fallback: self.fallback,
            },
            JoinPolicy {
                iterator: right,
                fallback: self.fallback,
            },
        )
    }
}

impl<I: ParallelIterator> ParallelIterator for JoinPolicy<I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential()
    }
}
