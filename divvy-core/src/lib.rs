#![no_std]

use core::{
    alloc::Layout,
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

#[derive(Debug)]
pub struct AllocError;

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

pub unsafe trait Deallocator {
    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    /// - The pointer must be valid and the same as given by a previous call to
    /// `allocate`.
    /// - The layout must be identical to that used when allocating the pointer.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout);

    /// Attempt to shrink a block of memory in-place.
    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        Err(AllocError)
    }

    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}

pub unsafe trait Allocator: Deallocator {
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError>;

    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.allocate(layout)?;
        unsafe { ptr.as_ptr().write_bytes(0, layout.size()) };
        Ok(ptr)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        if self.try_grow(ptr, old_layout, new_layout).is_ok() {
            return Ok(ptr);
        }

        let new = self.allocate(new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), old_layout.size());
        self.deallocate(ptr, old_layout);

        Ok(new)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
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

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        if self.try_shrink(ptr, old_layout, new_layout).is_ok() {
            return Ok(ptr);
        }

        let new = self.allocate(new_layout)?;
        ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr(), new_layout.size());
        self.deallocate(ptr, old_layout);

        Ok(new)
    }

    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        Err(AllocError)
    }

    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        self.try_grow(ptr, old_layout, new_layout)?;

        ptr.as_ptr()
            .add(old_layout.size())
            .write_bytes(0, new_layout.size() - old_layout.size());

        Ok(())
    }
}

unsafe impl<'a, A> Deallocator for &'a A
where
    A: Deallocator + ?Sized,
{
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        (**self).deallocate(ptr, layout)
    }

    unsafe fn try_shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        (**self).try_shrink(ptr, old_layout, new_layout)
    }
}

unsafe impl<'a, A> Allocator for &'a A
where
    A: Allocator + ?Sized,
{
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        (**self).allocate(layout)
    }

    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        (**self).allocate_zeroed(layout)
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        (**self).grow(ptr, old_layout, new_layout)
    }

    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        (**self).grow_zeroed(ptr, old_layout, new_layout)
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<NonNull<u8>, AllocError> {
        (**self).shrink(ptr, old_layout, new_layout)
    }

    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        (**self).try_grow(ptr, old_layout, new_layout)
    }

    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Result<(), AllocError> {
        (**self).try_grow_zeroed(ptr, old_layout, new_layout)
    }
}
