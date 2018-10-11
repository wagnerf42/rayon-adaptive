#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;
use criterion::Criterion;
use rayon_adaptive::*;

const NUM_THREADS: usize = 7;

fn infix_solver_bench(c: &mut Criterion) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(NUM_THREADS)
        .build_global()
        .expect("Rayon global pool initialisation failed");
    c.bench_function("adaptive infix (size=4_000_000)", |b| {
        b.iter_with_setup(
            || vec_gen(),
            |testin| {
                solver_adaptive(&testin, Policy::Adaptive(1000));
                testin
            },
        )
    });
    c.bench_function("parallel split infix (size=4_000_000)", |b| {
        b.iter_with_setup(
            || vec_gen(),
            |testin| {
                solver_par_split(&testin);
                testin
            },
        )
    });
    c.bench_function("sequential infix (size=4_000_000)", |b| {
        b.iter_with_setup(
            || vec_gen(),
            |testin| {
                solver_seq(&testin);
                testin
            },
        )
    });
    c.bench_function("parallel fold infix (size=4_000_000)", |b| {
        b.iter_with_setup(
            || vec_gen(),
            |testin| {
                solver_par_fold(&testin);
                testin
            },
        )
    });
}
criterion_group!(benches, infix_solver_bench);
criterion_main!(benches);
