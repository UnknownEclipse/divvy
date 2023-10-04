extern crate alloc;

use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc};
use core::{
    alloc::{GlobalAlloc, Layout},
    cmp,
    ptr::{self, NonNull},
};

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

#[derive(Debug, Default)]
pub struct WrapAsGlobal<A> {
    allocator: A,
}

impl<A> WrapAsGlobal<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator }
    }

    pub fn get_ref(&self) -> &A {
        &self.allocator
    }

    pub fn get_mut(&mut self) -> &mut A {
        &mut self.allocator
    }

    pub fn into_inner(self) -> A {
        self.allocator
    }
}

unsafe impl<A> GlobalAlloc for WrapAsGlobal<A>
where
    A: Allocator,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(layout) = NonZeroLayout::new(layout) {
            self.allocator
                .allocate(layout)
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null_mut())
        } else {
            layout.align() as *mut u8
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let Some(layout) = NonZeroLayout::new(layout) else {
            return;
        };
        let Some(ptr) = NonNull::new(ptr) else { return };
        unsafe { self.allocator.deallocate(ptr, layout) };
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if let Some(layout) = NonZeroLayout::new(layout) {
            self.allocator
                .allocate_zeroed(layout)
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null_mut())
        } else {
            layout.align() as *mut u8
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let old_layout = NonZeroLayout::new(layout);
        let Ok(new_layout) = Layout::from_size_align(new_size, layout.align()) else {
            return ptr::null_mut();
        };

        let new_layout = NonZeroLayout::new(new_layout);

        match (old_layout, new_layout) {
            (None, None) => ptr,
            (None, Some(new_layout)) => self
                .allocator
                .allocate(new_layout)
                .map(|p| p.as_ptr())
                .unwrap_or(ptr::null_mut()),
            (Some(old_layout), None) => {
                if let Some(ptr) = NonNull::new(ptr) {
                    self.allocator.deallocate(ptr, old_layout);
                }
                layout.align() as *mut u8
            }
            (Some(old_layout), Some(new_layout)) => {
                let ptr = unsafe { NonNull::new_unchecked(ptr) };

                let result = match old_layout.size().cmp(&new_layout.size()) {
                    cmp::Ordering::Less => self.allocator.grow(ptr, old_layout, new_layout),
                    cmp::Ordering::Equal => Ok(ptr),
                    cmp::Ordering::Greater => self.allocator.shrink(ptr, old_layout, new_layout),
                };

                result.map(|p| p.as_ptr()).unwrap_or(ptr::null_mut())
            }
        }
    }
}
