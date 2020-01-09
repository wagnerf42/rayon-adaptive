use rayon_adaptive::prelude::*;

fn main() {
    let some_vec: Vec<u32> = (0..1_000_000).collect();

    some_vec[0..1_000_000]
        .wrap_iter()
        .with_join_policy(1_000_000 / 8)
        .for_each(|s| {
            println!("{:?}", s.into_iter().collect::<Vec<_>>());
        });
}
