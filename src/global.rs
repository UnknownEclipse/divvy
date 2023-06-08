use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc, GlobalAlloc, Layout};
use core::{
    cmp::Ordering,
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocate, Deallocate, Grow, NonZeroLayout, Shrink};

/// An interface to Rust's current `#[global_allocator]`. All operations will be
/// transferred to functions in the `alloc` crate. Freeing pointers allocated by Rust
/// with this allocator is safe, as is the opposite.
#[derive(Debug, Default, Clone, Copy)]
pub struct Global;

unsafe impl Allocate for Global {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { alloc(layout.get()) };
        NonNull::new(ptr).ok_or(AllocError)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { alloc_zeroed(layout.get()) };
        NonNull::new(ptr).ok_or(AllocError)
    }
}

unsafe impl Grow for Global {
    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { realloc(ptr.as_ptr(), old_layout.get(), new_layout.size()) };
        NonNull::new(ptr).ok_or(AllocError)
    }
}

unsafe impl Shrink for Global {
    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        let ptr = unsafe { realloc(ptr.as_ptr(), old_layout.get(), new_layout.size()) };
        NonNull::new(ptr).ok_or(AllocError)
    }
}

unsafe impl Deallocate for Global {
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { dealloc(ptr.as_ptr(), layout.get()) };
    }
}

/// Wrap an allocator to implement the `GlobalAlloc` trait, allowing it to be used
/// as the global allocator.
#[derive(Debug, Default)]
pub struct WrapAsGlobal<A> {
    inner: A,
}

impl<A> WrapAsGlobal<A> {
    pub const fn new(alloc: A) -> Self {
        Self { inner: alloc }
    }

    pub fn into_inner(self) -> A {
        self.inner
    }

    pub fn get(&self) -> &A {
        &self.inner
    }
}

unsafe impl<A> GlobalAlloc for WrapAsGlobal<A>
where
    A: Allocate + Deallocate + Grow + Shrink,
{
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(layout) = NonZeroLayout::new(layout) {
            self.inner
                .allocate(layout)
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null_mut())
        } else {
            layout.align() as *mut u8
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if let (Some(ptr), Some(layout)) = (NonNull::new(ptr), NonZeroLayout::new(layout)) {
            unsafe { self.inner.deallocate(ptr, layout) }
        }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if let Some(layout) = NonZeroLayout::new(layout) {
            self.inner
                .allocate_zeroed(layout)
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null_mut())
        } else {
            layout.align() as *mut u8
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        let old_layout = NonZeroLayout::new(layout);

        let new_layout = match Layout::from_size_align(new_size, layout.align()) {
            Ok(layout) => NonZeroLayout::new(layout),
            Err(_) => return ptr::null_mut(),
        };

        let result = match (old_layout, new_layout) {
            (None, None) => return ptr.as_ptr(),
            (None, Some(new_layout)) => self.inner.allocate(new_layout),
            (Some(old_layout), None) => {
                unsafe { self.inner.deallocate(ptr, old_layout) };
                return layout.align() as *mut u8;
            }
            (Some(old_layout), Some(new_layout)) => {
                match old_layout.size().cmp(&new_layout.size()) {
                    Ordering::Less => self.inner.grow(ptr, old_layout, new_layout),
                    Ordering::Equal => return ptr.as_ptr(),
                    Ordering::Greater => self.inner.shrink(ptr, old_layout, new_layout),
                }
            }
        };

        result.map(|p| p.as_ptr()).unwrap_or(ptr::null_mut())
    }
}
