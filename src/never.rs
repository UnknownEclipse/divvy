use core::ptr::NonNull;

use divvy_core::{AllocError, Allocator, Deallocator, NonZeroLayout};

#[derive(Debug, Default, Clone)]
pub struct Never;

impl Deallocator for Never {
    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: NonZeroLayout) {
        unreachable!();
    }
}

unsafe impl Allocator for Never {
    #[inline]
    fn allocate(&self, _layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        Err(AllocError)
    }
}
