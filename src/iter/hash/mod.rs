//! adaptive iterators on hashmaps

use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
mod toxic; //don't open this
use self::toxic::{extract_hashmap_slices, extract_hashset_slices};

pub trait AdaptiveHashMap<'a> {
    type Iterator;
    fn adapt_keys(&'a self) -> Self::Iterator;
}

pub fn par_keys<'a, K: Send + Sync + Eq + Hash, V: Send + Sync>(
    hashmap: &'a HashMap<K, V>,
) -> impl AdaptiveIterator<Item = &'a K> {
    let (hashes, pairs) = unsafe { extract_hashmap_slices(hashmap) };
    hashes
        .into_adapt_iter()
        .zip(pairs.into_adapt_iter())
        .filter(|&(&h, _)| h != 0)
        .map(|(_, &(ref k, _))| k)
}

pub fn par_iter<'a, K: Send + Sync + Eq + Hash, V: Send + Sync>(
    hashmap: &'a HashMap<K, V>,
) -> impl AdaptiveIterator<Item = (&'a K, &'a V)> {
    let (hashes, pairs) = unsafe { extract_hashmap_slices(hashmap) };
    hashes
        .into_adapt_iter()
        .zip(pairs.into_adapt_iter())
        .filter(|&(&h, _)| h != 0)
        .map(|(_, &(ref k, ref v))| (k, v))
}

pub fn par_elements<'a, K: Send + Sync + Eq + Hash>(
    hashset: &'a HashSet<K>,
) -> impl AdaptiveIterator<Item = &'a K> {
    let (hashes, pairs) = unsafe { extract_hashset_slices(hashset) };
    hashes
        .into_adapt_iter()
        .zip(pairs.into_adapt_iter())
        .filter(|&(&h, _)| h != 0)
        .map(|(_, &(ref k, _))| k)
}
