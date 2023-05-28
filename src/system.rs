use std::{mem, ptr::NonNull};

use cfg_if::cfg_if;
use divvy_core::{AllocError, Allocate, Deallocate, Grow, NonZeroLayout, Shrink};
use libc::{c_void, size_t};

use crate::defaults::default_realloc;

/// The system allocator.
///
/// This is equivalent to `malloc()` on all platforms, and it is safe to free
/// pointers allocated by C libraries with this allocator and vice versa. Depending
/// on the platform specifics, a combination of malloc, realloc, posix_memalign,
/// and aligned_alloc are used alongside the trusted free function.
#[derive(Debug, Default, Clone, Copy)]
pub struct System;

impl System {
    #[inline]
    unsafe fn realloc(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        if new_layout.align() <= max_align() {
            let ptr = unsafe { libc::realloc(ptr.as_ptr().cast(), new_layout.size()) };
            NonNull::new(ptr.cast()).ok_or(AllocError)
        } else {
            default_realloc(self, ptr, old_layout, new_layout, false)
        }
    }
}

unsafe impl Allocate for System {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        extern "C" {
            fn aligned_alloc(align: size_t, size: size_t) -> *mut c_void;
        }

        let ptr = if layout.align() <= max_align() {
            unsafe { libc::malloc(layout.size()) }
        } else {
            let layout = layout.get().pad_to_align();
            unsafe { aligned_alloc(layout.align(), layout.size()) }
            // let mut ptr = MaybeUninit::uninit();
            // let rc =
            //     unsafe { libc::posix_memalign(ptr.as_mut_ptr(), layout.align(), layout.size()) };

            // if rc != 0 {
            //     Err(AllocError)
            // } else {
            //     let ptr = unsafe { ptr.assume_init().cast() };
            //     NonNull::new(ptr).ok_or(AllocError)
            // }
        };

        NonNull::new(ptr.cast()).ok_or(AllocError)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        if layout.align() <= max_align() {
            let ptr = unsafe { libc::calloc(layout.size(), 1) };
            NonNull::new(ptr.cast()).ok_or(AllocError)
        } else {
            let ptr = self.allocate(layout)?;
            unsafe {
                ptr.as_ptr().write_bytes(0, layout.size());
            }
            Ok(ptr)
        }
    }
}

unsafe impl Deallocate for System {
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, _layout: NonZeroLayout) {
        unsafe { libc::free(ptr.as_ptr().cast()) };
    }
}

unsafe impl Grow for System {
    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        self.realloc(ptr, old_layout, new_layout)
    }
}

unsafe impl Shrink for System {
    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        self.realloc(ptr, old_layout, new_layout)
    }
}

const fn max_align() -> usize {
    cfg_if! {
        if #[cfg(windows)] {
            mem::align_of::<f64>()
        } else {
            mem::align_of::<libc::max_align_t>()
        }
    }
}
