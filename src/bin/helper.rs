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
        .fold(Vec::new, |mut v, e| {
            v.push(f(e));
            v
        })
        .helping_fold(
            (),
            |_, i, limit| {
                let (todo, remaining) = i.divide_at(limit);
                for e in todo {
                    println!("{}", f(e));
                }
                ((), remaining)
            },
            |_, v| {
                for e in v {
                    println!("{}", e);
                }
                ()
            },
        );
}
