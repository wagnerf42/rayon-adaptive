use rayon_adaptive::prelude::*;

fn main() {
    let some_vec: Vec<u32> = (0..64).collect();

    some_vec[0..64]
        .par_iter()
        .wrap_iter()
        .with_join_policy(16)
        .for_each(|s| {
            println!("{:?}", s.into_iter().collect::<Vec<_>>());
        });
}
