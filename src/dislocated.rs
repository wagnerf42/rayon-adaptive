//! We provide a `Dislocated` type which will unsafely consider a *&'a I* so that *I* and *'a* are not
//! related, thus dropping the *'a* requirement on *I*.
//! This allows us to satisfy the ParallelIterator HRTB without sacrificing to convenience nor
//! security.
//! `DislocatedMut` is the equivalent type for mutable references.

use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

pub(crate) struct Dislocated<'a, I> {
    raw: *const I,
    phantom: PhantomData<&'a ()>,
}

impl<'a, I> Clone for Dislocated<'a, I> {
    fn clone(&self) -> Self {
        Dislocated {
            raw: self.raw,
            phantom: PhantomData,
        }
    }
}

impl<'a, I> Copy for Dislocated<'a, I> {}

impl<'a, I> Deref for Dislocated<'a, I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }.unwrap()
    }
}

impl<'a, I> Dislocated<'a, I> {
    pub(crate) fn new(input: &'a I) -> Dislocated<'a, I> {
        Dislocated {
            raw: input as *const I,
            phantom: PhantomData,
        }
    }
}

unsafe impl<'a, I: Sync> Send for Dislocated<'a, I> {}
unsafe impl<'a, I: Sync> Sync for Dislocated<'a, I> {}

pub(crate) struct DislocatedMut<'a, I> {
    raw: *mut I,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a, I> Deref for DislocatedMut<'a, I> {
    type Target = I;
    fn deref(&self) -> &Self::Target {
        unsafe { self.raw.as_ref() }.unwrap()
    }
}

impl<'a, I> DerefMut for DislocatedMut<'a, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.raw.as_mut() }.unwrap()
    }
}

impl<'a, I> DislocatedMut<'a, I> {
    pub(crate) fn new(input: &'a mut I) -> DislocatedMut<'a, I> {
        DislocatedMut {
            raw: input as *mut I,
            phantom: PhantomData,
        }
    }
    pub(crate) fn borrow_mut<'e>(&'e mut self) -> DislocatedMut<'e, I> {
        DislocatedMut {
            raw: self.raw,
            phantom: PhantomData,
        }
    }
}

unsafe impl<'a, I: Sync> Send for DislocatedMut<'a, I> {}
unsafe impl<'a, I: Sync> Sync for DislocatedMut<'a, I> {}
