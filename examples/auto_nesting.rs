use rayon_adaptive::prelude::*;
fn main() {
    let some_vec: Vec<u32> = (0..1000).collect();
    // Let's say I want 5 parallel iterators of 20 elements each, where the 20 elements are
    // processed sequentially
    (&some_vec[0..128])
        .into_par_iter()
        .wrap_iter()
        .with_join_policy(16)
        .for_each(|s| {
            println!("{:?}", s.into_iter().collect::<Vec<_>>());
        });
}
