extern crate criterion;
extern crate rayon;
extern crate rayon_adaptive;

use rand::Rng;
use rayon_adaptive::prelude::*;
use rayon_adaptive::Policy;
use std::fs::File;
use std::io::Write;
use std::iter::once;

fn main() -> std::io::Result<()> {
    let mut file = File::create("all.data")?;

    let iters: u64 = 500;
    let input_size = 50_000_000u64;
    // Setup
    // Execute the routine "iters" times
    for _ in 0..iters {
        let random_breakpoint = rand::thread_rng().gen_range(0, input_size);
        let start_par = time::precise_time_ns();
        (0u64..input_size)
            .into_par_iter()
            .with_policy(Policy::Adaptive(10_000, 50_000))
            .all(|e| e != random_breakpoint);
        let elapsed_par = time::precise_time_ns();
        let start_par_once = time::precise_time_ns();
        (0u64..input_size)
            .into_par_iter()
            .with_policy(Policy::Adaptive(10_000, 50_000))
            .by_blocks(once(input_size as usize))
            .all(|e| e != random_breakpoint);

        let elapsed_par_once = time::precise_time_ns();
        let start_seq = time::precise_time_ns();
        assert!(!(0u64..input_size).all(|e| e != random_breakpoint));
        let elapsed_seq = time::precise_time_ns();
        let speedup_par = (elapsed_seq - start_seq) as f64 / (elapsed_par - start_par) as f64;
        let speedup_par_once =
            (elapsed_seq - start_seq) as f64 / (elapsed_par_once - start_par_once) as f64;
        let random_bp_ratio = ((random_breakpoint * 100) as f64) / input_size as f64;
        file.write_all(
            format!(
                "{}\t{}\t{}\n",
                random_bp_ratio, speedup_par, speedup_par_once
            )
            .as_bytes(),
        )?;
    }
    Ok(())
}
