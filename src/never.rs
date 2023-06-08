use core::ptr::NonNull;

use divvy_core::{AllocError, Allocate, Deallocate, Grow, NonZeroLayout, Shrink};

#[derive(Debug, Default, Clone, Copy)]
pub struct Never;

unsafe impl Allocate for Never {
    #[inline]
    fn allocate(&self, _layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        Err(AllocError)
    }
}

unsafe impl Grow for Never {
    #[inline]
    unsafe fn grow(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: NonZeroLayout,
        _new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unreachable!("growth of unallocated memory");
    }
}

unsafe impl Shrink for Never {
    #[inline]
    unsafe fn shrink(
        &self,
        _ptr: NonNull<u8>,
        _old_layout: NonZeroLayout,
        _new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unreachable!("shrink of unallocated memory");
    }
}

unsafe impl Deallocate for Never {
    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: NonZeroLayout) {
        unreachable!("deallocation of unallocated pointer");
    }
}
