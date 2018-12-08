use std::cmp::min;
///! macro loop on input.
use DivisibleIntoBlocks;

pub struct Chunks<I: DivisibleIntoBlocks, S: Iterator<Item = usize>> {
    pub(crate) remaining: I,
    pub(crate) remaining_sizes: S,
}

impl<I: DivisibleIntoBlocks, S: Iterator<Item = usize>> Iterator for Chunks<I, S> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.len() == 0 {
            None
        } else {
            let next_size = min(
                self.remaining_sizes
                    .next()
                    .expect("not enough sizes for chunks"),
                self.remaining.len(),
            );
            let next_chunk = self.remaining.cut_left_at(next_size);
            Some(next_chunk)
        }
    }
}
