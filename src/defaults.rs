use std::{
    cmp,
    ptr::{self, NonNull},
};

use divvy_core::Grow;

use crate::{AllocError, Allocate, Deallocate, NonZeroLayout};

pub struct GrowRealloc {}

#[derive(Debug, Default)]
pub struct DefaultGrow<A> {
    inner: A,
}

impl<A> DefaultGrow<A> {
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

unsafe impl<A> Allocate for DefaultGrow<A>
where
    A: Allocate,
{
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate(layout)
    }

    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.inner.allocate_zeroed(layout)
    }
}

unsafe impl<A> Grow for DefaultGrow<A>
where
    A: Allocate + Deallocate,
{
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        default_realloc(&self.inner, ptr, old_layout, new_layout, false)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        default_realloc(&self.inner, ptr, old_layout, new_layout, true)
    }
}

unsafe impl<A> Deallocate for DefaultGrow<A>
where
    A: Deallocate,
{
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { self.inner.deallocate(ptr, layout) };
    }
}

pub unsafe fn default_realloc<A>(
    alloc: A,
    ptr: NonNull<u8>,
    old_layout: NonZeroLayout,
    new_layout: NonZeroLayout,
    zeroed: bool,
) -> Result<NonNull<u8>, AllocError>
where
    A: Allocate + Deallocate,
{
    let new = if zeroed {
        alloc.allocate_zeroed(new_layout)?
    } else {
        alloc.allocate(new_layout)?
    };

    let src = ptr.as_ptr();
    let dst = new.as_ptr();
    let n = cmp::min(old_layout.size(), new_layout.size());

    unsafe {
        ptr::copy_nonoverlapping(src, dst, n);
    }

    alloc.deallocate(ptr, old_layout);
    Ok(new)
}
