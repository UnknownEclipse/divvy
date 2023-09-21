#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    alloc::Layout,
    fmt::Display,
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

/// An error occurred during allocation, and the requested memory blocks could not
/// be returned.
#[derive(Debug)]
pub struct AllocError;

impl Display for AllocError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("allocation failed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AllocError {}

/// A `Layout` such that the size is never zero.
///
/// The distinction between zero sized layouts and any other layout is made because it
/// makes little sense for an allocator to concern itself with zero sized allocations.
/// That should be managed by the user of an allocator. This also saves a branch as
/// the allocator doesn't need to special case a zero sized allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonZeroLayout {
    layout: Layout,
}

impl NonZeroLayout {
    pub fn new(layout: Layout) -> Option<Self> {
        if layout.size() == 0 {
            None
        } else {
            Some(Self { layout })
        }
    }

    pub fn nonzero_size(&self) -> NonZeroUsize {
        let size = self.layout.size();
        unsafe { NonZeroUsize::new_unchecked(size) }
    }

    pub fn size(&self) -> usize {
        self.nonzero_size().get()
    }

    pub fn align(&self) -> usize {
        self.get().align()
    }

    pub fn get(&self) -> Layout {
        self.layout
    }
}

/// A `Deallocator` can be used to deallocate or shrink an allocation described by a
/// `NonZeroLayout`.
pub trait Deallocator {
    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    /// - The pointer must be valid and the same as given by a previous call to
    /// `allocate`.
    /// - The layout must be identical to that used when allocating the pointer.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout);

    /// Attempt to shrink a block of memory in-place.
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size <= old_size
    /// and new_align <= old_align.
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        let _ = ptr;
        let _ = old_layout;
        let _ = new_layout;
        Err(AllocError)
    }

    /// Creates a “by reference” adapter for this instance of `Dellocator`.
    /// The returned adapter also implements `Deallocator` and will simply borrow this.
    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}

/// # Safety
///
/// Allocator implementations require that allocated pointers are not invalidated
/// by moving the allocator. In other words, it is not valid to return a pointer
/// that references the address of self, such as returning the address of a member
/// array of bytes.
///
/// Allocators must ensure that allocated blocks do not overlap and remain valid for
/// the lifetime of the allocator or until a call to deallocate.
///
/// Allocators must also guarantee that returned memory blocks fit the requested layout.
pub unsafe trait Allocator: Deallocator {
    /// Allocate a new block of memory that fits the provided layout.
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError>;

    /// Allocate a new block of zeroed memory that fits the provided layout.
    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.allocate(layout)?;
        unsafe { ptr.as_ptr().write_bytes(0, layout.size()) };
        Ok(ptr)
    }

    /// Grow a previously allocated block of memory. If this call succeeds, the old
    /// pointer must not be used. If this call fails, the old pointer remains valid.
    ///
    /// For an alternative that guarantees that the pointer remains the same, see
    /// [try_grow](Self::try_grow).
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size >= old_size.
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            if self.try_grow(ptr, old_layout, new_layout).is_ok() {
                return Ok(ptr);
            }

            let new = self.allocate(new_layout)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);

            Ok(new)
        }
    }

    /// Grow a previously allocated block of memory, zeroing the newly allocated region.
    /// If this call succeeds, the old pointer must not be used. If this call fails, the
    /// old pointer remains valid.
    ///
    /// For an alternative that guarantees that the pointer remains the same, see
    /// [try_grow_zeroed](Self::try_grow_zeroed).
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size >= old_size.
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            if self.try_grow_zeroed(ptr, old_layout, new_layout).is_ok() {
                return Ok(ptr);
            }

            let new = self.allocate(new_layout)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), old_layout.size());
            self.deallocate(ptr, old_layout);

            ptr.as_ptr()
                .add(old_layout.size())
                .write_bytes(0, new_layout.size() - old_layout.size());

            Ok(new)
        }
    }

    /// Shrink a previously allocated block of memory. If this call succeeds, the old
    /// pointer must not be used. If this call fails, the old pointer remains valid.
    ///
    /// For an alternative that guarantees that the pointer remains the same, see
    /// [try_shrink](Self::try_shrink).
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size >= old_size.
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            if self.try_shrink(ptr, old_layout, new_layout).is_ok() {
                return Ok(ptr);
            }

            let new = self.allocate(new_layout)?;
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), new_layout.size());
            self.deallocate(ptr, old_layout);

            Ok(new)
        }
    }

    /// Attempt to grow a block of memory in-place.
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size >= old_size.
    #[inline]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        let _ = ptr;
        let _ = old_layout;
        let _ = new_layout;
        Err(AllocError)
    }

    /// Attempt to grow a block of memory in-place, zeroing the newly allocated portion.
    ///
    /// # Safety
    /// - The pointer must be valid and the same as returned by a previous call to
    /// `allocate`.
    /// - The old layout must be identical to that used when allocating the pointer
    /// - The new layout must have a size and alignment such that new_size >= old_size.
    #[inline]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        unsafe {
            self.try_grow(ptr, old_layout, new_layout)?;

            ptr.as_ptr()
                .add(old_layout.size())
                .write_bytes(0, new_layout.size() - old_layout.size());
        }
        Ok(())
    }
}

impl<'a, A> Deallocator for &'a A
where
    A: Deallocator + ?Sized,
{
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { (**self).deallocate(ptr, layout) }
    }

    #[inline]
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        unsafe { (**self).try_shrink(ptr, old_layout, new_layout) }
    }
}

unsafe impl<'a, A> Allocator for &'a A
where
    A: Allocator + ?Sized,
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
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { (**self).grow(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { (**self).grow_zeroed(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        unsafe { (**self).shrink(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        unsafe { (**self).try_grow(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        unsafe { (**self).try_grow_zeroed(ptr, old_layout, new_layout) }
    }
}
