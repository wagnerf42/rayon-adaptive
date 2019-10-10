//! Parallel deterministic random numbers
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use rayon_adaptive::prelude::*;

fn main() {
    let seed = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        0, 0,
    ];

    let mut r: Vec<u32> = vec![0; 100];
    rayon_adaptive::skip(
        (0, ChaChaRng::from_seed(seed)),
        |(c, rng)| {
            *c += 1;
            rng.gen()
        },
        |(c, rng), size| {
            let mut right_rng = rng.clone();
            right_rng.set_word_pos((*c + size) as u128);
            (*c + size, right_rng)
        },
    )
    .zip(r.as_mut_slice())
    .for_each(|(s, d)| *d = s);

    let r2: Vec<u32> = (0..100)
        .scan(ChaChaRng::from_seed(seed), |rng, _| Some(rng.gen()))
        .collect();

    assert_eq!(r, r2);
}
