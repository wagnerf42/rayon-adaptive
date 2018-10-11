use {Divisible, DivisibleAtIndex, EdibleSlice, Policy};

//TODO: switch to iterators
struct FindingSlice<'a, T: 'a> {
    slice: EdibleSlice<'a, T>,
    result: Option<T>,
}

impl<'a, T: 'a + Send + Sync> Divisible for FindingSlice<'a, T> {
    fn len(&self) -> usize {
        self.slice.len()
    }
    fn split(self) -> (Self, Self) {
        let (left_slice, right_slice) = self.slice.split();
        let my_part = FindingSlice {
            slice: left_slice,
            result: self.result,
        };
        let his_part = FindingSlice {
            slice: right_slice,
            result: None,
        };
        (my_part, his_part)
    }
}

impl<'a, T: 'a + Send + Sync> DivisibleAtIndex for FindingSlice<'a, T> {
    fn split_at(self, index: usize) -> (Self, Self) {
        let (left_slice, right_slice) = self.slice.split_at(index);
        let my_part = FindingSlice {
            slice: left_slice,
            result: self.result,
        };
        let his_part = FindingSlice {
            slice: right_slice,
            result: None,
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
    };
    input
        .work(|mut slice, limit| {
            if slice.result.is_none() {
                slice.result = slice.slice.iter().take(limit).find(|e| f(e)).cloned();
            }
            slice
        }).map(|slice| slice.result)
        .by_blocks(1_000_000)
        .filter_map(|o| o)
        .next()
}
