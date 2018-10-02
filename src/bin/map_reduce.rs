extern crate rayon_adaptive;

use rayon_adaptive::{DivisibleAtIndex, Mergeable};

struct Sum(u32);
impl Mergeable for Sum {
    fn fuse(self, other: Self) -> Self {
        Sum(self.0 + other.0)
    }
}

fn main() {
    let v: Vec<u32> = (0..20_000).collect();
    let s = v.as_slice();
    let sum = s.map_reduce(|s| Sum(s.iter().sum())).0;
    assert_eq!(sum, 10_000 * 19_999);
}
