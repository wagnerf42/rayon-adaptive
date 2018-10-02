#[cfg(not(feature = "logs"))]
extern crate rayon;
extern crate rayon_adaptive;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
use rayon::ThreadPoolBuilder;
use rayon_adaptive::{Divisible, EdibleSlice, Policy};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

//TODO: switch to iterators
struct FindingSlice<'a> {
    slice: EdibleSlice<'a, u32>,
    result: Option<u32>,
    //result: Option<&'a u32>, //TODO: how to do that ???
    found: Arc<AtomicBool>,
    previous_worker_found: Option<Arc<AtomicBool>>,
}

impl<'a> Divisible for FindingSlice<'a> {
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

fn find_first(v: &[u32], target: u32, policy: Policy) -> Option<u32> {
    let input = FindingSlice {
        slice: EdibleSlice::new(v),
        result: None,
        found: Arc::new(AtomicBool::new(false)),
        previous_worker_found: None,
    };
    input.work(
        |mut slice, limit| {
            slice.result = slice
                .slice
                .iter()
                .take(limit)
                .find(|&i| *i == target)
                .cloned();
            if slice.result.is_some() {
                slice.found.store(true, Ordering::SeqCst)
            }
            slice
        },
        |slice| slice.result,
        |left, right| left.or(right),
        policy,
    )
}

fn main() {
    let v: Vec<u32> = (0..10_000_000).collect();

    let pool = ThreadPoolBuilder::new()
        .num_threads(4)
        .build()
        .expect("pool creation failed");
    #[cfg(feature = "logs")]
    {
        let (answer, log) = pool.install(|| find_first(&v, 4_800_000, Policy::Adaptive(10000)));
        log.save_svg("find_first.svg").expect("saving svg failed");
        assert_eq!(answer.unwrap(), 4_800_000);
    }
    #[cfg(not(feature = "logs"))]
    {
        let answer = pool.install(|| find_first(&v, 4_800_000, Policy::Adaptive(10000)));
        assert_eq!(answer.unwrap(), 4_800_000);
    }
}
