use core::ptr::NonNull;

use crate::{AllocError, Allocate, Deallocate, NonZeroLayout};

#[derive(Debug, Default)]
pub struct Leak<A> {
    inner: A,
}

impl<A> Leak<A> {
    pub const fn new(alloc: A) -> Self {
        Self { inner: alloc }
    }

    #[inline]
    pub fn into_inner(self) -> A {
        self.inner
    }
}

unsafe impl<A> Allocate for Leak<A>
where
    A: Allocate,
{
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate(layout)
    }
}

unsafe impl<A> Deallocate for Leak<A> {
    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: NonZeroLayout) {}
}
