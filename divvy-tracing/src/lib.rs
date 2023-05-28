use std::{fmt::Debug, ptr::NonNull};

use divvy_core::{AllocError, Allocate, Deallocate, Grow, NonZeroLayout, Shrink};
use tracing::instrument;

#[derive(Debug)]
pub struct Traced<A> {
    inner: A,
}

unsafe impl<A> Allocate for Traced<A>
where
    A: Allocate + Debug,
{
    #[inline]
    #[instrument]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate(layout)
    }

    #[inline]
    #[instrument]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate_zeroed(layout)
    }

    #[inline]
    #[instrument]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        self.inner.try_grow(ptr, old_layout, new_layout)
    }

    #[inline]
    #[instrument]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        self.inner.try_grow_zeroed(ptr, old_layout, new_layout)
    }
}

unsafe impl<A> Deallocate for Traced<A>
where
    A: Deallocate + Debug,
{
    #[inline]
    #[instrument]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        self.inner.deallocate(ptr, layout)
    }

    #[inline]
    #[instrument]
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        self.inner.try_shrink(ptr, old_layout, new_layout)
    }
}

unsafe impl<A> Grow for Traced<A>
where
    A: Grow + Debug,
{
    #[inline]
    #[instrument]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        self.inner.grow(ptr, old_layout, new_layout)
    }

    #[inline]
    #[instrument]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        self.inner.grow(ptr, old_layout, new_layout)
    }
}

unsafe impl<A> Shrink for Traced<A>
where
    A: Shrink + Debug,
{
    #[inline]
    #[instrument]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        self.inner.shrink(ptr, old_layout, new_layout)
    }
}
