use std::marker::PhantomData;

pub trait Splittable<'s>: Sized {
    /// What we split into
    type R: Splittable<'s>;
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R);
}

impl<'s, T> Splittable<'s> for &'s mut [T] {
    type R = &'s mut [T];
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R) {
        (self as &mut [T]).split_at_mut(mid)
    }
}

pub struct SplitZip<'z, SA: Splittable<'z>, SB: Splittable<'z>> {
    splittables: (SA, SB),
    phantom: PhantomData<&'z u32>, // i just want the lifetime
}

pub fn zip<'z, SA: Splittable<'z>, SB: Splittable<'z>>(sa: SA, sb: SB) -> SplitZip<'z, SA, SB> {
    SplitZip {
        splittables: (sa, sb),
        phantom: PhantomData,
    }
}

impl<'s, SA: Splittable<'s> + 's, SB: Splittable<'s> + 's> Splittable<'s> for SplitZip<'s, SA, SB> {
    type R = SplitZip<'s, <SA as Splittable<'s>>::R, <SB as Splittable<'s>>::R>;
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R) {
        let (left0, right0) = self.splittables.0.split_at_mut(mid);
        let (left1, right1) = self.splittables.1.split_at_mut(mid);
        (
            SplitZip {
                splittables: (left0, left1),
                phantom: PhantomData,
            },
            SplitZip {
                splittables: (right0, right1),
                phantom: PhantomData,
            },
        )
    }
}
