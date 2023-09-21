use core::{
    cell::Cell,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocator, Deallocator, NonZeroLayout};

use crate::sub_ptr;

#[derive(Debug)]
pub struct FixedSlice<'a> {
    data: NonNull<[u8]>,
    pos: Cell<NonNull<u8>>,
    _p: PhantomData<&'a mut [u8]>,
}

impl<'a> FixedSlice<'a> {
    /// Create a new allocator that allocates from the provided slice of memory.
    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        unsafe { Self::from_ptr_slice(slice) }
    }

    /// Create a new allocator that allocates from the provided slice of memory.
    pub fn from_uninit_slice(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        unsafe { Self::from_ptr_slice(slice as *mut [MaybeUninit<u8>] as *mut [u8]) }
    }

    /// Unsafely construct a fixed slice from a pointer to a block of memory.
    ///
    /// See the safe version, [from_slice](FixedSlice::from_slice) for more information.
    ///
    /// # Safety
    ///
    /// The pointer must exclusively reference a mutable slice that remains valid for
    /// the lifetime of the `FixedSlice`. In addition
    pub unsafe fn from_ptr_slice(slice: *mut [u8]) -> Self {
        let slice = NonNull::new_unchecked(slice);
        Self {
            data: slice,
            pos: Cell::new(slice.cast()),
            _p: PhantomData,
        }
    }

    /// Return a pointer to the portion of the slice that has yet to be allocated.
    pub fn unallocated_ptr(&self) -> NonNull<[u8]> {
        let data = self.pos.get().as_ptr();
        let end = {
            let start: *mut u8 = self.data.as_ptr().cast();
            let len = self.data.len();
            unsafe { start.add(len) }
        };
        let len = unsafe { sub_ptr(end, data) };
        let ptr = ptr::slice_from_raw_parts_mut(data, len);
        unsafe { NonNull::new_unchecked(ptr) }
    }
}

impl<'a> Deallocator for FixedSlice<'a> {
    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: divvy_core::NonZeroLayout) {}
}

unsafe impl<'a> Allocator for FixedSlice<'a> {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let bump_result =
            unsafe { bump_alloc_impl(self.data, self.pos.get(), layout).ok_or(AllocError)? };
        self.pos.set(bump_result.pos);
        Ok(bump_result.ptr)
    }
}

#[derive(Debug)]
struct BumpResult {
    ptr: NonNull<u8>,
    pos: NonNull<u8>,
}

unsafe fn bump_alloc_impl(
    arena: NonNull<[u8]>,
    pos: NonNull<u8>,
    layout: NonZeroLayout,
) -> Option<BumpResult> {
    let pos = pos.as_ptr();
    let offset = (pos as usize) - (arena.as_ptr() as *mut u8 as usize);
    let align_offset = pos.align_offset(layout.align());

    let end_offset = offset
        .checked_add(align_offset)?
        .checked_add(layout.size())?;

    if end_offset < arena.len() {
        unsafe {
            let ptr = pos.add(align_offset);
            let new_pos = pos.add(layout.size());

            let ptr = NonNull::new_unchecked(ptr);
            let new_pos = NonNull::new_unchecked(new_pos);

            Some(BumpResult { ptr, pos: new_pos })
        }
    } else {
        None
    }
}
