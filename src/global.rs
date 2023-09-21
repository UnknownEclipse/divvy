extern crate alloc;

use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc};
use core::ptr::NonNull;

use divvy_core::{AllocError, Allocator, Deallocator, NonZeroLayout};

#[derive(Debug, Default, Clone)]
pub struct Global;

impl Global {
    #[inline]
    unsafe fn realloc(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        let result = unsafe { realloc(ptr.as_ptr(), old_layout.get(), new_layout.size()) };
        NonNull::new(result).ok_or(AllocError)
    }
}

impl Deallocator for Global {
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { dealloc(ptr.as_ptr(), layout.get()) };
    }
}

unsafe impl Allocator for Global {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let result = unsafe { alloc(layout.get()) };
        NonNull::new(result).ok_or(AllocError)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let result = unsafe { alloc_zeroed(layout.get()) };
        NonNull::new(result).ok_or(AllocError)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { self.realloc(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { self.realloc(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { self.realloc(ptr, old_layout, new_layout) }
    }
}
