use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use self::unix::{os_alloc, os_alloc_zeroed, os_dealloc, os_grow, os_grow_zeroed, os_shrink};

mod unix;

#[derive(Debug, Default, Clone, Copy)]
pub struct Os;

unsafe impl Allocator for Os {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // This function is inlined so this check can be optimized out in most cases
        if layout.size() == 0 {
            let ptr = layout.dangling();
            Ok(NonNull::slice_from_raw_parts(ptr, 0))
        } else {
            os_alloc(layout)
        }
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // This function is inlined so this check can be optimized out in most cases
        if layout.size() != 0 {
            unsafe { os_dealloc(ptr, layout) };
        }
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() == 0 {
            let ptr = layout.dangling();
            Ok(NonNull::slice_from_raw_parts(ptr, 0))
        } else {
            os_alloc_zeroed(layout)
        }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        if new_layout.size() == old_layout.size() {
            Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
        } else {
            os_grow(ptr, old_layout, new_layout)
        }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        if new_layout.size() == old_layout.size() {
            Ok(NonNull::slice_from_raw_parts(ptr, new_layout.size()))
        } else {
            os_grow_zeroed(ptr, old_layout, new_layout)
        }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        os_shrink(ptr, old_layout, new_layout)
    }
}
