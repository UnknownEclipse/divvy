//! # Divvy
//!
//! An alternative allocators api for Rust projects.
//!
//! # Design Decisions
//!
//! ## Split Interface
//!
//! While the standard [Allocator](std::alloc::Allocator) trait is entirely self
//! contained, this crate splits the capabilities into their own traits. There are a
//! few reasons this decision was made.
//!
//! 1. Types that do not support deallocation will not pretend to deallocate memory,
//! which may lead to surprising leaks in the future. Instead, the user must explciitly
//! request a noop deallocation method via the [Leak] wrapper type.
//! 2. Many structs do not need to allocate any memory once they are created. This can
//! be statically enforced by only requiring [Deallocate] in their drop bounds
//! rather than a fully featured allocator. This also allows for schemes which have
//! different support for allocation and deallocation. For example, a mimalloc heap
//! can allocate only on its local thread, but can deallocate from any thread. This can
//! be statically modeled by having a shared handle that only implements [Deallocate].
//!
//! ## `NonZeroLayout`
//!
//! Zero-sized allocations are uniquely handled. Allocating methods accept only
//! NonZeroLayouts which enforce that a type is not zero-sized.

#![no_std]
#![allow(clippy::missing_safety_doc)]

use core::{alloc::Layout, fmt::Display, num::NonZeroUsize, ptr::NonNull};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonZeroLayout(Layout);

impl NonZeroLayout {
    #[inline]
    pub const fn new(layout: Layout) -> Option<Self> {
        if layout.size() == 0 {
            None
        } else {
            Some(Self(layout))
        }
    }

    #[inline]
    pub const unsafe fn new_unchecked(layout: Layout) -> Self {
        Self(layout)
    }

    #[inline]
    pub fn get(&self) -> Layout {
        self.0
    }

    #[inline]
    pub fn size(&self) -> usize {
        self.nonzero_size().get()
    }

    #[inline]
    pub fn nonzero_size(&self) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(self.0.size()) }
    }

    #[inline]
    pub fn align(&self) -> usize {
        self.0.align()
    }
}

#[derive(Debug)]
pub struct AllocError;

impl Display for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("allocation error")
    }
}

/// The base trait for all allocators.
///
/// This covers allocation of memory *only*.
///
/// # Safety
///
/// Pointers returned by every method must be unique, unaliased, and stable when
/// the allocator is moved.
pub unsafe trait Allocate {
    /// Allocate a block of memory.
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError>;

    /// Allocate a block of memory, ensuring that the memory is zeroed.
    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.allocate(layout)?;
        unsafe { ptr.as_ptr().write_bytes(0, layout.size()) };
        Ok(ptr)
    }

    /// Attempt to grow an allocation *in-place*. Implementars should not attempt
    /// to create a new allocation. For that, see the [Grow] trait. As a check,
    /// the returned pointer should always be equal to the pointer passed in.
    ///
    /// # Safety
    /// 1. The pointer must be valid and have been returned by a previous call
    /// to this allocator.
    /// 2. The old layout must be the same as the pointer was originally allocated
    /// with.
    /// 3. The new layout must be at least as large as the old layout.
    #[inline]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        _ = (ptr, old_layout, new_layout);
        None
    }

    /// Attempt to grow an allocation *in-place*, ensuring the newly allocated
    /// portion is zeroed. Implementars should not attempt to create a new allocation.
    /// For that, see the [Grow] trait. As a check, the returned pointer should always
    /// be equal to the pointer passed in.
    ///
    /// # Safety
    /// 1. The pointer must be valid and have been returned by a previous call
    /// to this allocator.
    /// 2. The old layout must be the same as the pointer was originally allocated
    /// with.
    /// 3. The new layout must be at least as large as the old layout.
    #[inline]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        let ptr = self.try_grow(ptr, old_layout, new_layout)?;

        unsafe {
            ptr.as_ptr()
                .add(old_layout.size())
                .write_bytes(0, new_layout.size() - old_layout.size());
        }

        Some(ptr)
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError>
    where
        Self: Deallocate,
    {
        todo!()
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError>
    where
        Self: Deallocate,
    {
        todo!()
    }
}

/// The deallocation half of an allocator. This allows for deallocation of pointers
/// previously allocated.
pub unsafe trait Deallocate {
    /// Deallocate a pointer.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout);

    /// Attempt to shrink an allocation *in-place*. Implementations must not attempt
    /// to create a new, smaller allocation. For that, see the [Shrink] trait.
    #[inline]
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        _ = (ptr, old_layout, new_layout);
        None
    }
}

unsafe impl<'a, A> Allocate for &'a A
where
    A: Allocate + ?Sized,
{
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        (**self).allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        (**self).allocate_zeroed(layout)
    }

    #[inline]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        unsafe { (**self).try_grow(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        unsafe { (**self).try_grow_zeroed(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError>
    where
        Self: Deallocate,
    {
        unsafe { (**self).grow(ptr, old_layout, new_layout) }
    }
}

unsafe impl<'a, A> Deallocate for &'a A
where
    A: Deallocate + ?Sized,
{
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { (**self).deallocate(ptr, layout) };
    }

    #[inline]
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        unsafe { (**self).try_shrink(ptr, old_layout, new_layout) }
    }
}
