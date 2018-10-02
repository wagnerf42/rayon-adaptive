extern crate rayon_adaptive;
use rayon_adaptive::*;
#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
extern crate time;

use rayon::ThreadPoolBuilder;
const NUM_THREADS: usize = 3;

fn main() {
    let random_expression = vec_gen();
    let pool = ThreadPoolBuilder::new()
        .num_threads(NUM_THREADS)
        .build()
        .expect("Pool creation failed");

    let answer = solver_seq(&random_expression);

    #[cfg(feature = "logs")]
    {
        pool.compare(
            "sequential",
            "adaptive",
            || {
                let count = solver_seq(&random_expression);
                assert_eq!(count, answer);
            },
            || {
                let count = solver_adaptive(&random_expression, Policy::Adaptive(1000));
                assert_eq!(count, answer);
            },
            "seq_adapt.html",
        ).expect("logging failed");

        pool.compare(
            "adaptive",
            "rayon split",
            || {
                let count = solver_adaptive(&random_expression, Policy::Adaptive(1000));
                assert_eq!(count, answer);
            },
            || {
                let count = solver_par_split(&random_expression);
                assert_eq!(count, answer);
            },
            "adapt_split.html",
        ).expect("logging failed");
        pool.compare(
            "adaptive",
            "rayon fold",
            || {
                let count = solver_adaptive(&random_expression, Policy::Adaptive(1000));
                assert_eq!(count, answer);
            },
            || {
                let count = solver_par_fold(&random_expression);
                assert_eq!(count, answer);
            },
            "adapt_fold.html",
        ).expect("logging failed");
    }

    #[cfg(feature = "logs")]
    {
        let count = solver_adaptive(&random_expression, Policy::Adaptive(1000));
        assert_eq!(count, answer);
    }
}
