use crate::prelude::*;
use crate::traits::BlockedPower;
use itertools::Itertools;
use std::str::Chars;

/// Adaptive iterator on characters of strings.
pub struct AdaptiveChars<'a> {
    real_str: &'a str,
}

impl<'a> IntoIterator for AdaptiveChars<'a> {
    type Item = char;
    type IntoIter = Chars<'a>;
    fn into_iter(self) -> Self::IntoIter {
        self.real_str.chars()
    }
}

impl<'a> AdaptiveChars<'a> {
    fn find_splitting_index_around(&self, start_index: usize) -> usize {
        let len = self.real_str.len();
        let higher_indices = start_index..len;
        let lower_indices = (0..start_index).rev();
        lower_indices
            .interleave(higher_indices)
            .find(|&i| self.real_str.is_char_boundary(i))
            .unwrap_or(len) // if we cannot split we just generate an empty slice
    }
}

impl<'a> Divisible for AdaptiveChars<'a> {
    type Power = BlockedPower;
    fn base_length(&self) -> usize {
        self.real_str.len()
    }
    /// Pre-condition: self.can_be_divided() is true.
    fn divide(self) -> (Self, Self) {
        let index = self.find_splitting_index_around(self.real_str.len() / 2);
        let (left, right) = self.real_str.split_at(index);
        (
            AdaptiveChars { real_str: left },
            AdaptiveChars { real_str: right },
        )
    }
}

impl<'a> DivisibleIntoBlocks for AdaptiveChars<'a> {
    /// Pre-condition: self.can_be_divided() is true.
    fn divide_at(self, index: usize) -> (Self, Self) {
        let index = self.find_splitting_index_around(index);
        let (left, right) = self.real_str.split_at(index);
        (
            AdaptiveChars { real_str: left },
            AdaptiveChars { real_str: right },
        )
    }
}

impl<'a> AdaptiveIterator for AdaptiveChars<'a> {}

pub trait AdaptiveString {
    fn adapt_chars(&self) -> AdaptiveChars;
}

impl AdaptiveString for str {
    fn adapt_chars(&self) -> AdaptiveChars {
        AdaptiveChars { real_str: self }
    }
}
