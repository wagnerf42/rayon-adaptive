extern crate rayon_adaptive;
use rayon_adaptive::*;
extern crate rayon_logs;
extern crate time;
use rayon_logs::ThreadPoolBuilder;
const NUM_THREADS: usize = 3;
fn main() {
    let testin = vec_gen();
    let pool = ThreadPoolBuilder::new()
        .num_threads(NUM_THREADS)
        .build()
        .expect("Pool creation failed");

    let answer = solver_seq(&testin);

    pool.compare(
        "sequential",
        "adaptive",
        || {
            let count = solver_seq(&testin);
            assert_eq!(count, answer);
        },
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        "seq_adapt.html",
    ).expect("logging failed");

    pool.compare(
        "adaptive",
        "rayon split",
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        || {
            let count = solver_par_split(&testin);
            assert_eq!(count, answer);
        },
        "adapt_split.html",
    ).expect("logging failed");
    pool.compare(
        "adaptive",
        "rayon fold",
        || {
            let count = solver_adaptive(&testin, Policy::Adaptive(1000));
            assert_eq!(count, answer);
        },
        || {
            let count = solver_par_fold(&testin);
            assert_eq!(count, answer);
        },
        "adapt_fold.html",
    ).expect("logging failed");
}
