//! Adaptive prefix algorithm.
//! No macro blocks.
use {Divisible, EdibleSliceMut, Policy};

/// Run adaptive prefix algortihm on given slice.
/// Each element is replaced by folding with op since beginning of the slice.
/// It requires an associative operation.
///
/// # Example
///
/// ```
/// use rayon_adaptive::adaptive_prefix;
/// let mut v = vec![1u32; 100_000];
/// adaptive_prefix(&mut v, |e1, e2| e1 + e2);
/// let count: Vec<u32> = (1..=100_000).collect();
/// assert_eq!(v, count);
/// ```
pub fn adaptive_prefix<T, O>(v: &mut [T], op: O)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    let input = EdibleSliceMut::new(v);
    input
        .work(|mut slice, limit| {
            let c = {
                let mut elements = slice.iter_mut().take(limit);
                let mut c = elements.next().unwrap().clone();
                for e in elements {
                    *e = op(e, &c);
                    c = e.clone();
                }
                c
            };
            // pre-update next one
            if let Some(e) = slice.peek() {
                *e = op(e, &c);
            }
            slice
        }).map(|slice| slice.slice())
        .into_iter()
        .fold(
            None,
            |potential_previous_slice: Option<&mut [T]>, current_slice| {
                if let Some(previous_slice) = potential_previous_slice {
                    update(current_slice, previous_slice.last().cloned().unwrap(), &op);
                }
                Some(current_slice)
            },
        );
}

fn update<T, O>(slice: &mut [T], increment: T, op: &O)
where
    T: Send + Sync + Clone,
    O: Fn(&T, &T) -> T + Sync,
{
    {
        let input = EdibleSliceMut::new(slice);
        input
            .with_policy(Policy::Adaptive(1000))
            .for_each(|mut s, limit| {
                for e in s.iter_mut().take(limit) {
                    *e = op(e, &increment)
                }
                s
            });
    }
}
