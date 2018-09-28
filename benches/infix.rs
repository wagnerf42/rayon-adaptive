#[macro_use]
extern crate criterion;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;
use criterion::Criterion;
use rayon_adaptive::*;

fn infix_solver_bench(c: &mut Criterion) {
    c.bench_function("adaptive infix (size=4_000_000)", |b| {
        b.iter_with_setup(
            || vec_gen(),
            |testin| solver_adaptive(&testin, Policy::Adaptive(1000)),
        )
    });
    c.bench_function("parallel split infix (size=4_000_000)", |b| {
        b.iter_with_setup(|| vec_gen(), |testin| solver_par_split(&testin))
    });
    c.bench_function("sequential infix (size=4_000_000)", |b| {
        b.iter_with_setup(|| vec_gen(), |testin| solver_seq(&testin))
    });
    c.bench_function("parallel fold infix (size=4_000_000)", |b| {
        b.iter_with_setup(|| vec_gen(), |testin| solver_par_fold(&testin))
    });
}
criterion_group!(benches, infix_solver_bench);
criterion_main!(benches);
