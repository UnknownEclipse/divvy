use core::ffi::c_void;
use std::ptr::NonNull;

use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct NewDelete;

unsafe impl Allocate for NewDelete {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { divvy_cpp_alloc(layout.size(), layout.align()) };
        NonNull::new(ptr.cast()).ok_or(AllocError)
    }
}

unsafe impl Deallocate for NewDelete {
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { divvy_cpp_dealloc(ptr.as_ptr().cast(), layout.size(), layout.align()) };
    }
}

#[allow(non_camel_case_types)]
type c_size_t = usize;

extern "C" {
    pub fn divvy_cpp_alloc(size: c_size_t, align: c_size_t) -> *mut c_void;
    pub fn divvy_cpp_dealloc(ptr: *mut c_void, size: c_size_t, align: c_size_t);
}
