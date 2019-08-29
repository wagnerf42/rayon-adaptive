struct ParSuccessors<T, F, S> {
    next: T,
    succ: F,
    skip_op: S,
}

struct BoundedParSuccessors<'a, T, F, S> {
    next: T,
    remaining_iterations: usize,
    succ: F,
    skip_op: S,
    real_iterator_next: Option<&'a mut T>,
}

struct SeqSuccessors<'a, T: Clone, F> {
    next: T,
    succ: F,
    real_iterator_next: Option<&'a mut T>,
}

impl<
        'extraction,
        T: 'static + Send + Clone,
        F: Fn(T) -> T + Clone + Send,
        S: Send + Clone + Fn(T, usize) -> T,
    > ExtractiblePart<'extraction, T> for ParSuccessors<T, F, S>
{
    type BorrowedPart = BoundedParSuccessors<'extraction, T, F, S>;
}

impl<T, F, S> Extractible<T> for ParSuccessors<T, F, S>
where
    T: Clone + 'static + Send,
    F: Fn(T) -> T + Clone + Send,
    S: Fn(T, usize) -> T + Clone + Send,
{
    fn borrow_on_left_for<'extraction>(
        &'extraction mut self,
        size: usize,
    ) -> <Self as ExtractiblePart<'extraction, T>>::BorrowedPart {
        BoundedParSuccessors {
            next: self.next.clone(),
            remaining_iterations: size,
            succ: self.succ.clone(),
            skip_op: self.skip_op.clone(),
            real_iterator_next: Some(&mut self.next),
        }
    }
}

impl<'a, T, F> Iterator for SeqSuccessors<'a, T, F>
where
    T: Clone,
    F: Fn(T) -> T + Clone,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        let next_next = (self.succ)(self.next.clone());
        let current_next = std::mem::replace(&mut self.next, next_next);
        Some(current_next)
    }
}

impl<'a, T, F> Drop for SeqSuccessors<'a, T, F>
where
    T: Clone,
{
    fn drop(&mut self) {
        if let Some(real_next) = &mut self.real_iterator_next {
            **real_next = self.next.clone()
        }
    }
}

impl<'a, T, F, S> ParallelIterator for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone + Send,
    F: Fn(T) -> T + Clone + Send,
    S: Fn(T, usize) -> T + Clone + Send,
{
    type Item = T;
    type SequentialIterator = Take<SeqSuccessors<'a, T, F>>;
    fn len(&self) -> usize {
        self.remaining_iterations
    }
    fn to_sequential(self) -> Self::SequentialIterator {
        SeqSuccessors {
            next: self.next,
            succ: self.succ,
            real_iterator_next: self.real_iterator_next,
        }
        .take(self.remaining_iterations)
    }
}

impl<'a, T, F, S> Divisible for BoundedParSuccessors<'a, T, F, S>
where
    T: Clone,
    F: Fn(T) -> T + Clone,
    S: Fn(T, usize) -> T + Clone,
{
    fn is_divisible(&self) -> bool {
        self.remaining_iterations > 1
    }
    fn divide(mut self) -> (Self, Self) {
        let mid = self.remaining_iterations / 2;
        let right_next = (self.skip_op)(self.next.clone(), mid);
        let right = BoundedParSuccessors {
            next: right_next,
            remaining_iterations: self.remaining_iterations - mid,
            succ: self.succ.clone(),
            skip_op: self.skip_op.clone(),
            real_iterator_next: self.real_iterator_next.take(),
        };
        self.remaining_iterations = mid;
        (self, right)
    }
}


