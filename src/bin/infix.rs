extern crate rayon_adaptive;
use rayon_adaptive::*;
#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
extern crate time;
use rayon::ThreadPoolBuilder;
const NUM_THREADS: usize = 8;

fn main() {
    #[cfg(feature = "logs")]
    {
        let pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("Pool creation failed");

        pool.compare()
            .attach_algorithm_with_setup(
                "sequential",
                || vec_gen(),
                |vec| {
                    solver_seq(&vec);
                    vec
                },
            ).attach_algorithm_with_setup(
                "adaptive",
                || vec_gen(),
                |vec| {
                    solver_adaptive(&vec, Policy::Adaptive(1000));
                    vec
                },
            ).attach_algorithm_with_setup(
                "rayon split",
                || vec_gen(),
                |vec| {
                    solver_par_split(&vec);
                    vec
                },
            ).attach_algorithm_with_setup(
                "rayon fold",
                || vec_gen(),
                |vec| {
                    solver_par_fold(&vec);
                    vec
                },
            ).generate_logs("comparisons.html")
            .expect("comparison failed");
    }
    #[cfg(not(feature = "logs"))]
    {
        ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build_global()
            .expect("Pool creation failed");

        let testin = vec_gen();
        let answer = solver_seq(&testin);
        let adapt_ans = solver_adaptive(&testin, Policy::Adaptive(1000));
        let parsplit_ans = solver_par_split(&testin);
        let parfold_ans = solver_par_fold(&testin);
        assert_eq!(answer, adapt_ans);
        assert_eq!(answer, parsplit_ans);
        assert_eq!(answer, parfold_ans);
    }
}
