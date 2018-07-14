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
    let mut x: Vec<u32> = (0..100).collect();
    let s2 = x.as_mut_slice();
    let z = zip(s0, s1);
    let mut z2 = zip(z, s2);
    let (zl, zr) = z2.split_at_mut(50);

    //sort(&mut v);
}
