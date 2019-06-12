extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rand::Rng;
use rayon_adaptive::prelude::*;
use std::fs::File;
use std::io::Write;
use itertools::enumerate;
const iters: usize = 100;

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
    let mut filescheduling = File::create("allscheduling.data")?;
    let input_size = vec![
        1_000, 10_000, 50_000, 100_000, 1_000_000, 2_000_000, 5_000_000, 10_000_000, 25_000_000,
        50_000_000,
    ];

    for (j,size) in enumerate(input_size) {
        let mut vecall = [0f64; iters as usize];
        let mut vecallscheduling = [0f64; iters as usize];
        for i in 0..iters {
            
            let random_breakpoint = rand::thread_rng().gen_range(0, size);
            let start_seq = time::precise_time_ns();
            assert!(!(0u64..size).all(|e| e != random_breakpoint));
            let elapsed_seq = time::precise_time_ns();
            let start_time = time::precise_time_ns();
            (0u64..size)
                .into_par_iter()
                .all(|e| e != random_breakpoint);
            let end_time = time::precise_time_ns();

            let mut speedup_par_once =
                (elapsed_seq - start_seq) as f64 / (end_time - start_time) as f64;
            vecall[i] = speedup_par_once;
            let start_time = time::precise_time_ns();
            (0u64..size)
                .into_par_iter()
                .allscheduling(|e| e != random_breakpoint);
            let end_time = time::precise_time_ns();
            
            speedup_par_once =
                (elapsed_seq - start_seq) as f64 / (end_time - start_time) as f64;
            vecallscheduling[i] = speedup_par_once;
        }
        vecall.sort_by(|a, b| a.partial_cmp(b).unwrap());
            file.write_all(
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    j,
                    min(vecall),
                    q1(vecall),
                    median(vecall),
                    q3(vecall),
                    max(vecall),
                    0.3,
                    size,
                    average(vecall)
                )
                .as_bytes(),
            )?;
        vecallscheduling.sort_by(|a, b| a.partial_cmp(b).unwrap());
            filescheduling.write_all(
                format!(
                    "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                    j,
                    min(vecallscheduling),
                    q1(vecallscheduling),
                    median(vecallscheduling),
                    q3(vecallscheduling),
                    max(vecallscheduling),
                    0.3,
                    size,
                    average(vecallscheduling)
                )
                .as_bytes(),
            )?;
           
    }
     Ok(())
}