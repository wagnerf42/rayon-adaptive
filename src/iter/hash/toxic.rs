use prelude::*;
use std::cmp::max;
use std::collections::HashMap;
use std::hash::Hash;
use std::marker;
use std::mem::transmute; // from the depths of hell, I summon you

type HashUint = usize;

// An unsafe view of a RawTable bucket
// Valid indexes are within [0..table_capacity)
pub struct RawBucket<K, V> {
    hash_start: *mut HashUint,
    // We use *const to ensure covariance with respect to K and V
    pair_start: *const (K, V),
    idx: usize,
    _marker: marker::PhantomData<(K, V)>,
}

/// A raw iterator. The basis for some other iterators in this module. Although
/// this interface is safe, it's not used outside this module.
struct RawBuckets<'a, K, V> {
    raw: RawBucket<K, V>,
    elems_left: usize,

    // Strictly speaking, this should be &'a (K,V), but that would
    // require that K:'a, and we often use RawBuckets<'static...> for
    // move iterations, so that messes up a lot of other things. So
    // just use `&'a (K,V)` as this is not a publicly exposed type
    // anyway.
    marker: marker::PhantomData<&'a ()>,
}

/// Iterator over shared references to entries in a table.
pub struct Iter<'a, K: 'a, V: 'a> {
    iter: RawBuckets<'a, K, V>,
}

// RawBucket methods are unsafe as it's possible to
// make a RawBucket point to invalid memory using safe code.
impl<K, V> RawBucket<K, V> {
    unsafe fn hash(&self) -> *mut HashUint {
        self.hash_start.add(self.idx)
    }
    unsafe fn pair(&self) -> *mut (K, V) {
        self.pair_start.add(self.idx) as *mut (K, V)
    }
    unsafe fn hash_pair(&self) -> (*mut HashUint, *mut (K, V)) {
        (self.hash(), self.pair())
    }
}

const EMPTY_BUCKET: HashUint = 0;
impl<K, V> Copy for RawBucket<K, V> {}
impl<K, V> Clone for RawBucket<K, V> {
    fn clone(&self) -> RawBucket<K, V> {
        *self
    }
}

const MIN_NONZERO_RAW_CAPACITY: usize = 32; // must be a power of two
/// A hash map's "capacity" is the number of elements it can hold without
/// being resized. Its "raw capacity" is the number of slots required to
/// provide that capacity, accounting for maximum loading. The raw capacity
/// is always zero or a power of two.
#[inline]
fn try_raw_capacity(len: usize) -> Option<usize> {
    if len == 0 {
        Some(0)
    } else {
        // 1. Account for loading: `raw_capacity >= len * 1.1`.
        // 2. Ensure it is a power of two.
        // 3. Ensure it is at least the minimum size.
        len.checked_mul(11)
            .map(|l| l / 10)
            .and_then(|l| l.checked_next_power_of_two())
            .map(|l| max(MIN_NONZERO_RAW_CAPACITY, l))
    }
}

#[inline]
fn raw_capacity(len: usize) -> usize {
    try_raw_capacity(len).expect("raw_capacity overflow")
}

/// Parallel Iterator over shared references to entries in a table.
pub struct ParBuckets<'a, K: 'a, V: 'a> {
    iter: RawBuckets<'a, K, V>,
    max_index: usize,
}

impl<'a, K, V> Iterator for ParBuckets<'a, K, V> {
    type Item = RawBucket<K, V>;

    fn next(&mut self) -> Option<RawBucket<K, V>> {
        while self.iter.raw.idx != self.max_index {
            //while self.iter.elems_left != 0 {
            unsafe {
                let item = self.iter.raw;
                self.iter.raw.idx += 1;
                if *item.hash() != EMPTY_BUCKET {
                    println!("found at {}/{}", self.iter.raw.idx - 1, self.max_index);
                    self.iter.elems_left -= 1;
                    return Some(item);
                }
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

struct ParIter<'a, K: 'a, V: 'a> {
    iter: ParBuckets<'a, K, V>,
}

impl<'a, K, V> Iterator for ParIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        self.iter.next().map(|raw| unsafe {
            let pair_ptr = raw.pair();
            (&(*pair_ptr).0, &(*pair_ptr).1)
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

unsafe fn extract_slices<'a, K: Eq + Hash, V>(
    table: &'a HashMap<K, V>,
) -> (&'a [HashUint], &'a [(K, V)]) {
    let capacity = raw_capacity(table.capacity());
    let i: std::collections::hash_map::Iter<'a, K, V> = table.iter();
    // I feel like satan himself
    let i = transmute::<std::collections::hash_map::Iter<'a, K, V>, Iter<'a, K, V>>(i);
    let hashes = std::slice::from_raw_parts(i.iter.raw.hash_start, capacity);
    let pairs = std::slice::from_raw_parts(i.iter.raw.pair_start, capacity);
    (hashes, pairs)
}

fn keys<'a, K: Eq + Hash, V>(table: &'a HashMap<K, V>) -> impl Iterator<Item = &'a K> {
    let (hashes, pairs) = unsafe { extract_slices(table) };
    hashes
        .iter()
        .zip(pairs.iter())
        .filter_map(|(h, p)| if *h == EMPTY_BUCKET { None } else { Some(&p.0) })
}
