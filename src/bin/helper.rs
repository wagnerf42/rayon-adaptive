use rayon_adaptive::prelude::*;

fn f(e: usize) -> usize {
    let mut c = 0;
    for x in 0..e {
        c += x;
    }
    c
}

fn main() {
    (0..10_000)
        .into_adapt_iter()
        .map(|e| f(e))
        .fold(Vec::new, |mut v, e| {
            v.push(e);
            v
        })
        .helping_for_each(
            |e| println!("{}", e),
            |v| {
                for e in v {
                    println!("{}", e);
                }
            },
        );
}
