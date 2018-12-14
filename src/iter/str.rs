use crate::prelude::*;
use crate::traits::BlockedPower;
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

impl<'a> Divisible for AdaptiveChars<'a> {
    type Power = BlockedPower;
    fn base_length(&self) -> usize {
        self.real_str.len()
    }
    fn divide(self) -> (Self, Self) {
        // TODO: this is not safe if called with a size too small
        // we need to change the trait to return an option.
        let mut index = self.real_str.len() / 2;
        while !self.real_str.is_char_boundary(index) {
            index += 1;
        }
        let (left, right) = self.real_str.split_at(index);
        (
            AdaptiveChars { real_str: left },
            AdaptiveChars { real_str: right },
        )
    }
}

impl<'a> DivisibleIntoBlocks for AdaptiveChars<'a> {
    fn divide_at(self, mut index: usize) -> (Self, Self) {
        // TODO: this is not safe if called with a size too small
        // we need to change the trait to return an option.
        while !self.real_str.is_char_boundary(index) {
            index += 1;
        }
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
