use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct Never;

unsafe impl Allocator for Never {
    #[inline]
    fn allocate(&self, _layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError)
    }

    #[inline]
    #[track_caller]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        unreachable!("attempted to free unallocated pointer");
    }
}
