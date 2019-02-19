use rayon_adaptive::prelude::*;
const TEST_NUM: usize = 5184;
fn main() {
    (0..TEST_NUM).into_adapt_iter().for_each(|num| {
        assert!(num >= 0);
    });
}
