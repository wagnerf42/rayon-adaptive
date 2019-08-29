struct RayonPolicy<I> {
    iterator: I,
    counter: usize,
    created_by: Option<usize>,
}

impl<I: ParallelIterator> Divisible for RayonPolicy<I> {
    fn is_divisible(&self) -> bool {
        self.iterator.is_divisible() && self.counter != 0
    }
    fn divide(self) -> (Self, Self) {
        let (left, right) = self.iterator.divide();
        let current_thread = rayon::current_thread_index();
        let new_counter = if current_thread == self.created_by {
            self.counter - 1
        } else {
            (rayon::current_num_threads() as f64).log(2.0).ceil() as usize
        };
        (
            RayonPolicy {
                iterator: left,
                counter: new_counter,
                created_by: current_thread,
            },
            RayonPolicy {
                iterator: right,
                counter: new_counter,
                created_by: current_thread,
            },
        )
    }
}

impl<I: ParallelIterator> ParallelIterator for RayonPolicy<I> {
    type Item = I::Item;
    type SequentialIterator = I::SequentialIterator;
    fn len(&self) -> usize {
        self.iterator.len()
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        self.iterator.to_sequential()
    }
}


