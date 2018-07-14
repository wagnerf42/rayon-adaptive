extern crate rayon_adaptive;
use rayon_adaptive::slices::{zip, Splittable};

fn sort(s: &mut [u32]) {
    unimplemented!();
}

fn main() {
    let mut v: Vec<u32> = (0..100).collect();
    let s0 = v.as_mut_slice();
    let mut w: Vec<u32> = (0..100).collect();
    let s1 = w.as_mut_slice();
    let mut z = zip(s0, s1);
    let (zl, zr) = z.split_at_mut(50);

    //sort(&mut v);
}
