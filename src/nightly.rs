use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::{self, NonNull},
};

use divvy_core::{Allocate, Deallocate, Grow, NonZeroLayout, Shrink};

#[derive(Debug, Default)]
pub struct AsStd<A> {
    inner: A,
}

impl<A> AsStd<A> {
    pub const fn new(alloc: A) -> Self {
        Self { inner: alloc }
    }

    #[inline]
    pub fn into_inner(self) -> A {
        self.inner
    }
}

unsafe impl<A> Allocator for AsStd<A>
where
    A: Allocate + Deallocate + Grow + Shrink,
{
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = match NonZeroLayout::new(layout) {
            Some(layout) => self
                .inner
                .allocate(layout)
                .map_err(|_| AllocError)?
                .as_ptr(),
            None => layout.align() as *mut u8,
        };

        Ok(NonNull::new(ptr::slice_from_raw_parts_mut(ptr, layout.size())).unwrap())
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        if let Some(layout) = NonZeroLayout::new(layout) {
            self.inner.deallocate(ptr, layout);
        }
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = match NonZeroLayout::new(layout) {
            Some(layout) => self
                .inner
                .allocate_zeroed(layout)
                .map_err(|_| AllocError)?
                .as_ptr(),
            None => layout.align() as *mut u8,
        };

        Ok(NonNull::new(ptr::slice_from_raw_parts_mut(ptr, layout.size())).unwrap())
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let result = match (
            NonZeroLayout::new(old_layout),
            NonZeroLayout::new(new_layout),
        ) {
            (Some(old_layout), Some(new_layout)) => self.inner.grow(ptr, old_layout, new_layout),
            (None, Some(layout)) => self.inner.allocate(layout),
            _ => unreachable!(),
        };

        result.map_err(|_| AllocError).map(|ptr| {
            let ptr = ptr.as_ptr();
            let slice = ptr::slice_from_raw_parts_mut(ptr, new_layout.size());
            NonNull::new(slice).unwrap()
        })
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let result = match (
            NonZeroLayout::new(old_layout),
            NonZeroLayout::new(new_layout),
        ) {
            (Some(old_layout), Some(new_layout)) => {
                self.inner.grow_zeroed(ptr, old_layout, new_layout)
            }
            (None, Some(layout)) => self.inner.allocate_zeroed(layout),
            _ => unreachable!(),
        };

        result.map_err(|_| AllocError).map(|ptr| {
            let ptr = ptr.as_ptr();
            let slice = ptr::slice_from_raw_parts_mut(ptr, new_layout.size());
            NonNull::new(slice).unwrap()
        })
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = match (
            NonZeroLayout::new(old_layout),
            NonZeroLayout::new(new_layout),
        ) {
            (Some(old_layout), Some(new_layout)) => self
                .inner
                .shrink(ptr, old_layout, new_layout)
                .map_err(|_| AllocError)?
                .as_ptr(),
            (Some(layout), None) => {
                unsafe { self.inner.deallocate(ptr, layout) };
                new_layout.align() as *mut u8
            }
            _ => unreachable!(),
        };

        let slice = ptr::slice_from_raw_parts_mut(ptr, new_layout.size());
        Ok(NonNull::new(slice).unwrap())
    }
}
