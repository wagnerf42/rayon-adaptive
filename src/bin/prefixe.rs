use rayon_adaptive::adaptive_prefix;

fn main() {
    let mut v = vec![1.0; 1_000_000];
    let res = v.clone();
    let start = time::precise_time_ns();
    adaptive_prefix(&mut v, |e1, e2| e1 * e2);
    let end = time::precise_time_ns();
    assert_eq!(v, res);
    println!("we did it in {}", ((end - start) as f64) / (1e6 as f64));

    let mut v = vec![1.0; 1_000_000];
    let res = v.clone();
    let start = time::precise_time_ns();
    v.iter_mut().fold(1.0, |c, e| {
        *e *= c;
        *e
    });
    let end = time::precise_time_ns();
    assert_eq!(v, res);
    println!(
        "sequential did it in {}",
        ((end - start) as f64) / (1e6 as f64)
    );
}
