#[macro_use]
extern crate criterion;
extern crate itertools;
extern crate rand;
extern crate rayon;
extern crate rayon_adaptive;

use itertools::Itertools;
use rand::prelude::*;
use rayon_adaptive::prelude::*;

use criterion::{Criterion, ParameterizedBenchmark};

fn safe_manual_merge(left: &[u32], right: &[u32], output: &mut [u32]) {
    let sizes = [left.len(), right.len()];
    let inputs = [left, right];
    let mut indices = [0usize; 2];
    for o in output {
        let direction = if indices[0] >= sizes[0] {
            1
        } else if indices[1] >= sizes[1] {
            0
        } else if inputs[0][indices[0]] >= inputs[1][indices[1]] {
            0
        } else {
            1
        };
        *o = inputs[direction][indices[direction]];
        indices[direction] += 1;
    }
}

fn unsafe_manual_merge(left: &[u32], right: &[u32], output: &mut [u32]) {
    let sizes = [left.len(), right.len()];
    let inputs = [left, right];
    let mut indices = [0usize; 2];
    for o in output {
        let direction = if indices[0] >= sizes[0] {
            1
        } else if indices[1] >= sizes[1] {
            0
        } else if inputs[0][indices[0]] >= inputs[1][indices[1]] {
            0
        } else {
            1
        };
        *o = unsafe { *inputs[direction].get_unchecked(indices[direction]) };
        indices[direction] += 1;
    }
}

fn interleaved_input(input_size: u32) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let (mut left, mut right): (Vec<_>, Vec<_>) = (0..input_size).tuples().unzip();
    let output = vec![0u32; input_size as usize];
    let mut rng = thread_rng();
    left.shuffle(&mut rng);
    right.shuffle(&mut rng);
    (left, right, output)
}

fn merge_benchmarks(c: &mut Criterion) {
    let sizes: Vec<u32> = vec![100_000, 500_000, 1_000_000, 2_000_000];
    // let sizes: Vec<u32> = vec![100_000, 1_000_000, 10_000_000, 50_000_000, 100_000_000];
    c.bench(
        "merge (random input, interleaved, shuffled)",
        ParameterizedBenchmark::new(
            "itertool merge",
            |b, input_size| {
                b.iter_with_setup(
                    || interleaved_input(*input_size),
                    |(left, right, mut output)| {
                        left.iter()
                            .merge(right.iter())
                            .zip(output.iter_mut())
                            .for_each(|(i, o)| *o = *i);
                        (left, right, output)
                    },
                )
            },
            sizes.clone(),
        )
        .with_function("safe manual merge", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    safe_manual_merge(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
        .with_function("unsafe manual merge", |b, input_size| {
            b.iter_with_setup(
                || interleaved_input(*input_size),
                |(left, right, mut output)| {
                    unsafe_manual_merge(&left, &right, &mut output);
                    (left, right, output)
                },
            )
        })
        .with_function("adaptive generic merge", |b, input_size| {
            b.iter_with_setup(
                || {
                    let thread_pool = rayon::ThreadPoolBuilder::new()
                        .num_threads(1)
                        .build()
                        .expect("Thread pool creation failed!");
                    (thread_pool, interleaved_input(*input_size))
                },
                |(tp, (left, right, mut output))| {
                    // TODO: I wonder if the install is not taking time here
                    tp.install(|| {
                        left.par_iter()
                            .merge(right.par_iter())
                            .directional_zip(output.par_iter_mut())
                            .for_each(|(i, o)| *o = *i)
                    });
                    (left, right, output)
                },
            )
        }),
    );
}

criterion_group! {
    name = benches;
            config = Criterion::default().sample_size(10).nresamples(100);
                targets = merge_benchmarks
}
criterion_main!(benches);
