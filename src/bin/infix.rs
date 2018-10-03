extern crate rayon_adaptive;
use rayon_adaptive::*;
#[cfg(not(feature = "logs"))]
extern crate rayon;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;
extern crate time;
//use rayon_logs::Comparator;
use rayon::ThreadPoolBuilder;
const NUM_THREADS: usize = 3;
fn main() {
    #[cfg(feature = "logs")]
    {
        let testin = vec_gen();
        let pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("Pool creation failed");

        let answer = solver_seq(&testin);
        let comparator = pool.compare();
        let res = comparator
            .attach_algorithm_with_setup(
                String::from("sequential"),
                || vec_gen(),
                |vec| {
                    solver_seq(&vec);
                },
            )
            .attach_algorithm_with_setup(
                String::from("adaptive"),
                || vec_gen(),
                |vec| {
                    solver_adaptive(&vec, Policy::Adaptive(1000));
                },
            )
            .attach_algorithm(String::from("rayon split"), || {
                let count = solver_par_split(&testin);
                assert_eq!(count, answer);
            })
            .attach_algorithm(String::from("rayon fold"), || {
                let count = solver_par_fold(&testin);
                assert_eq!(count, answer);
            })
            .generate_logs("comparisons.html");
        res.expect("comparison failed");
    }
    #[cfg(feature = "logs")]
    {
        let testin = vec_gen();
        let pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build_global()
            .expect("Pool creation failed");

        let answer = solver_seq(&testin);
        let adapt_ans = solver_adaptive(&testin, Policy::Adaptive(1000));
        let parsplit_ans = solver_par_split(&testin);
        let parfold_ans = solver_par_fold(&testin);
        assert_eq!(answer, adapt_ans);
        assert_eq!(answer, parsplit_ans);
        assert_eq!(answer, parfold_ans);
    }
}
