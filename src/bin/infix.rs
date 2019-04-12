use rayon_adaptive::*;
use time::precise_time_ns;
#[cfg(feature = "logs")]
extern crate rayon_logs as rayon;

use rayon::ThreadPoolBuilder;
use std::iter::repeat_with;
const NUM_THREADS: usize = 2;
const SIZE: u64 = 1_000_000;

fn print_timing<U, I: Send + Sized, G: Fn() -> I, F: Fn(I) -> U>(label: &str, setup: G, f: F) {
    let time: u64 = repeat_with(setup)
        .take(100)
        .map(|v| {
            let start = precise_time_ns();
            f(v);
            let end = precise_time_ns();
            end - start
        })
        .sum();
    println!("{} : {}", label, time / 100);
}

fn main() {
    #[cfg(feature = "logs")]
    {
        let pool = ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build()
            .expect("Pool creation failed");

        pool.compare()
            .runs_number(100)
            .attach_algorithm_nodisplay_with_setup(
                "sequential",
                || vec_gen(SIZE),
                |vec| {
                    solver_seq(&vec);
                    vec
                },
            )
            .attach_algorithm_with_setup(
                "fully adaptive",
                || vec_gen(SIZE),
                |vec| {
                    solver_fully_adaptive(&vec);
                    vec
                },
            )
            .attach_algorithm_with_setup(
                "adaptive",
                || vec_gen(SIZE),
                |vec| {
                    solver_adaptive(&vec, Default::default());
                    vec
                },
            )
            .generate_logs(format!(
                "comparisons_{}mil_{}threads.html",
                SIZE / (1e6 as u64),
                NUM_THREADS
            ))
            .expect("comparison failed");
    }
    #[cfg(not(feature = "logs"))]
    {
        ThreadPoolBuilder::new()
            .num_threads(NUM_THREADS)
            .build_global()
            .expect("Pool creation failed");

        let random_expression = vec_gen(SIZE);
        let answer = solver_seq(&random_expression);
        let adapt_ans = solver_adaptive(&random_expression, Default::default());
        let parsplit_ans = solver_par_split(&random_expression);
        let parfold_ans = solver_par_fold(&random_expression);
        let adaptfold_ans = solver_fully_adaptive(&random_expression);
        assert_eq!(answer, adapt_ans);
        assert_eq!(answer, parsplit_ans);
        assert_eq!(answer, parfold_ans);
        assert_eq!(answer, adaptfold_ans);
        print_timing("sequential", || vec_gen(SIZE), |v| solver_seq(&v));
        print_timing("par split", || vec_gen(SIZE), |v| solver_par_split(&v));
        print_timing("par fold", || vec_gen(SIZE), |v| solver_par_fold(&v));
        print_timing(
            "adapt",
            || vec_gen(SIZE),
            |v| solver_adaptive(&v, Default::default()),
        );
        print_timing(
            "fully_adapt",
            || vec_gen(SIZE),
            |v| solver_fully_adaptive(&v),
        );
    }
}
