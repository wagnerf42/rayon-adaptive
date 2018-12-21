use rayon::join;
use rayon_adaptive::prelude::*;

fn f(e: usize) -> usize {
    let mut c = 0;
    for x in 0..e {
        c += x;
    }
    c
}

fn fold_with_help<I, O1, O2, ID2, FOLD1, FOLD2, RET>(
    i: I,
    o1: O1,
    fold1: FOLD1,
    id2: ID2,
    fold2: FOLD2,
    retrieve: RET,
) -> O1
where
    I: DivisibleIntoBlocks,
    ID2: Fn() -> O2 + Sync,
    O1: Send + Sync,
    O2: Send + Sync,
    FOLD1: Fn(O1, I, usize) -> (O1, I) + Sync,
    FOLD2: Fn(O2, I, usize) -> (O2, I) + Sync,
    RET: Fn(O1, O2) -> O1 + Sync,
{
    unimplemented!()
}

fn main() {
    fold_with_help(
        0..10_000,
        (),
        |_, i, limit| {
            let (todo, remaining) = i.divide_at(limit);
            for e in todo {
                println!("{}", f(e));
            }
            ((), remaining)
        },
        Vec::new,
        |mut v, i, limit| {
            let (todo, remaining) = i.divide_at(limit);
            v.extend(todo.into_iter().map(|e| f(e)));
            (v, remaining)
        },
        |_, v| {
            for e in v {
                println!("{}", e);
            }
            ()
        },
    )
}
