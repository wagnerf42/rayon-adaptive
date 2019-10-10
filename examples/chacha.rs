//! Parallel deterministic random numbers
use rand::prelude::*;
use rand_chacha::ChaChaRng;
use rayon_adaptive::prelude::*;

fn main() {
    let seed = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0,
        0, 0,
    ];
    let mut rng1 = ChaChaRng::from_seed(seed);
    let res: Vec<u32> = std::iter::repeat_with(|| rng1.gen()).take(10).collect();
    eprintln!("res: {:?}", res);

    let mut rng1 = ChaChaRng::from_seed(seed);
    let x: u32 = rng1.gen();
    let mut rng2 = rng1.clone();
    rng2.set_word_pos(1);
    let y: u32 = rng2.gen();
    rng2.set_word_pos(2);
    let res: Vec<u32> = std::iter::repeat_with(|| rng2.gen()).take(5).collect();
    eprintln!("res: {:?}", res);

    let mut r = vec![(0, 0, 0); 3];
    rayon_adaptive::skip(
        (0, ChaChaRng::from_seed(seed)),
        |(c, rng)| {
            *c += 1;
            (*c - 1, rng.gen(), rng.get_word_pos())
        },
        |(c, rng), size| {
            eprintln!(
                "ok, we already did: {}, someone is asking to skip {}",
                *c, size
            );
            let mut right_rng = rng.clone();
            right_rng.set_word_pos(*c + size as u128);
            eprintln!("setting to {}", *c + size as u128);
            (*c + size as u128, right_rng)
        },
    )
    .zip(r.as_mut_slice())
    .with_join_policy(10)
    .for_each(|(s, d)| *d = s);
    eprintln!("res: {:?}", r);
}
