extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rand::Rng;
use rayon_adaptive::prelude::*;
use std::fs::File;
use std::io::Write;

const iters: u64 = 1_000;

fn average(numbers: [f64; iters as usize]) -> f64 {
    numbers.iter().sum::<f64>() / numbers.len() as f64
}

fn median(numbers: [f64; iters as usize]) -> f64 {
    let mid = numbers.len() / 2;
    numbers[mid]
}
fn q1(numbers: [f64; iters as usize]) -> f64 {
    let mid = numbers.len() / 4;
    numbers[mid]
}
fn q3(numbers: [f64; iters as usize]) -> f64 {
    let mid = numbers.len() * 3 / 4;
    numbers[mid]
}

fn max(numbers: [f64; iters as usize]) -> f64 {
    numbers[numbers.len() - 1]
}

fn min(numbers: [f64; iters as usize]) -> f64 {
    numbers[0]
}

fn main() -> std::io::Result<()> {
    let mut file = File::create("all.data")?;
    let input_size = vec![
        1_000, 10_000, 50_000, 100_000, 1_000_000, 2_000_000, 5_000_000, 10_000_000, 25_000_000,
        50_000_000,
    ];
    // Setup
    // Execute the routine "iters" times
    let mut result = vec![[0f64; iters as usize]; input_size.len()];;
    for j in 0..iters {
        for i in 0..input_size.len() {
            let random_breakpoint = rand::thread_rng().gen_range(0, input_size[i]);
            let start_seq = time::precise_time_ns();
            assert!(!(0u64..input_size[i]).all(|e| e != random_breakpoint));
            let elapsed_seq = time::precise_time_ns();
            let start_time = time::precise_time_ns();
            (0u64..input_size[i])
                .into_par_iter()
                .all(|e| e != random_breakpoint);
            let end_time = time::precise_time_ns();

            let speedup_par_once =
                (elapsed_seq - start_seq) as f64 / (end_time - start_time) as f64;
            result[i as usize][j as usize] = speedup_par_once;
        }
    }
    for i in 0..result.len() {
        // Data columns: X Min 1stQuartile Median 3rdQuartile Max BoxWidth Titles
        result[i].sort_by(|a, b| a.partial_cmp(b).unwrap());
        file.write_all(
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                i,
                min(result[i]),
                q1(result[i]),
                median(result[i]),
                q3(result[i]),
                max(result[i]),
                0.3,
                input_size[i],
                average(result[i])
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
