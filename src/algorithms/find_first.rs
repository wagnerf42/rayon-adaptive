use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use {Divisible, EdibleSlice, Policy};

//TODO: switch to iterators
struct FindingSlice<'a, T: 'a> {
    slice: EdibleSlice<'a, T>,
    result: Option<T>,
    //result: Option<&'a u32>, //TODO: how to do that ???
    found: Arc<AtomicBool>,
    previous_worker_found: Option<Arc<AtomicBool>>,
}

impl<'a, T: 'a + Send + Sync> Divisible for FindingSlice<'a, T> {
    fn len(&self) -> usize {
        if self.found.load(Ordering::SeqCst) || self
            .previous_worker_found
            .as_ref()
            .map(|f| f.load(Ordering::SeqCst))
            .unwrap_or(false)
        {
            0
        } else {
            self.slice.len()
        }
    }
    fn split(self) -> (Self, Self) {
        let (left_slice, right_slice) = self.slice.split();
        let my_part = FindingSlice {
            slice: left_slice,
            result: None,
            found: self.found,
            previous_worker_found: self.previous_worker_found,
        };
        let his_part = FindingSlice {
            slice: right_slice,
            result: None,
            found: Arc::new(AtomicBool::new(false)),
            previous_worker_found: Some(my_part.found.clone()),
        };
        (my_part, his_part)
    }
}

/// Return first element for which f returns true.
pub fn find_first<T, F>(v: &[T], f: F, policy: Policy) -> Option<T>
where
    T: Sync + Send + Copy,
    F: Fn(&&T) -> bool + Sync,
{
    let input = FindingSlice {
        slice: EdibleSlice::new(v),
        result: None,
        found: Arc::new(AtomicBool::new(false)),
        previous_worker_found: None,
    };
    input
        .work(|mut slice, limit| {
            slice.result = slice.slice.iter().take(limit).find(|e| f(e)).cloned();
            if slice.result.is_some() {
                slice.found.store(true, Ordering::SeqCst)
            }
            slice
        }).map(|slice| slice.result)
        .reduce(|left, right| left.or(right), policy)
}
