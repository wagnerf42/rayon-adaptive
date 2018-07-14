use std::marker::PhantomData;

pub trait Splittable<'s>: Sized {
    /// What we split into
    type R;
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R);
}

impl<'s, T> Splittable<'s> for &'s mut [T] {
    type R = &'s mut [T];
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R) {
        (self as &mut [T]).split_at_mut(mid)
    }
}

pub struct SliceZip<'a, TA: 'a, TB: 'a> {
    slices: (&'a mut [TA], &'a mut [TB]),
}

pub fn zip<'a, TA: 'a, TB: 'a>(slicea: &'a mut [TA], sliceb: &'a mut [TB]) -> SliceZip<'a, TA, TB> {
    SliceZip {
        slices: (slicea, sliceb),
    }
}

impl<'s, TA, TB> Splittable<'s> for SliceZip<'s, TA, TB> {
    type R = SliceZip<'s, TA, TB>;
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R) {
        let (left0, right0) = self.slices.0.split_at_mut(mid);
        let (left1, right1) = self.slices.1.split_at_mut(mid);
        (
            SliceZip {
                slices: (left0, left1),
            },
            SliceZip {
                slices: (right0, right1),
            },
        )
    }
}

struct SplitZip<'z, SA: Splittable<'z>, SB: Splittable<'z>> {
    splittables: (SA, SB),
}

impl<'s, SA: Splittable<'s> + 's, SB: Splittable<'s> + 's> Splittable<'s> for SplitZip<'s, SA, SB> {
    type R = SplitZip<'s, SA, SB>;
    fn split_at_mut(&'s mut self, mid: usize) -> (Self::R, Self::R) {
        let (left0, right0) = self.splittables.0.split_at_mut(mid);
        let (left1, right1) = self.splittables.1.split_at_mut(mid);
        (
            SplitZip {
                splittables: (left0, left1),
            },
            SplitZip {
                splittables: (right0, right1),
            },
        )
    }
}
