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
    _idx: usize,
    _marker: marker::PhantomData<(K, V)>,
}

/// A raw iterator. The basis for some other iterators in this module. Although
/// this interface is safe, it's not used outside this module.
struct RawBuckets<'a, K, V> {
    raw: RawBucket<K, V>,
    _elems_left: usize,

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

pub(crate) unsafe fn extract_slices<'a, K: Eq + Hash, V>(
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
