use core::{
    cell::Cell,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

use divvy_core::{AllocError, Allocate, NonZeroLayout};

#[derive(Debug)]
pub struct Slice<'a> {
    slice: NonNull<[u8]>,
    pos: Cell<usize>,
    _p: PhantomData<&'a [u8]>,
    zeroed: bool,
}

impl<'a> Slice<'a> {
    #[inline]
    pub const unsafe fn new_unchecked(slice: NonNull<[u8]>, zeroed: bool) -> Self {
        Self {
            slice,
            pos: Cell::new(0),
            _p: PhantomData,
            zeroed,
        }
    }

    #[inline]
    pub fn new(slice: &'a mut [u8]) -> Self {
        unsafe { Self::new_unchecked(NonNull::from(slice), false) }
    }

    #[inline]
    pub fn new_uninit(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        let slice = NonNull::new(slice as *mut [MaybeUninit<u8>] as *mut [u8]).unwrap();
        unsafe { Self::new_unchecked(slice, false) }
    }
}

unsafe impl<'a> Allocate for Slice<'a> {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let pos = self.pos.get();
        let alloc = alloc(self.slice, pos, layout).ok_or(AllocError)?;
        self.pos.set(alloc.new_pos);
        Ok(alloc.ptr)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.allocate(layout)?;
        if !self.zeroed {
            unsafe { ptr.as_ptr().write_bytes(0, layout.size()) };
        }
        Ok(ptr)
    }
}

#[derive(Debug)]
pub struct SyncSlice<'a> {
    slice: NonNull<[u8]>,
    pos: AtomicUsize,
    _p: PhantomData<&'a [u8]>,
    zeroed: bool,
}

impl<'a> SyncSlice<'a> {
    #[inline]
    pub const unsafe fn new_unchecked(slice: NonNull<[u8]>, zeroed: bool) -> Self {
        Self {
            slice,
            pos: AtomicUsize::new(0),
            _p: PhantomData,
            zeroed,
        }
    }

    #[inline]
    pub fn new(slice: &'a mut [u8]) -> Self {
        unsafe { Self::new_unchecked(NonNull::from(slice), false) }
    }

    #[inline]
    pub fn new_uninit(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        let slice = NonNull::new(slice as *mut [MaybeUninit<u8>] as *mut [u8]).unwrap();
        unsafe { Self::new_unchecked(slice, false) }
    }
}

unsafe impl<'a> Allocate for SyncSlice<'a> {
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let mut ptr = NonNull::dangling();

        self.pos
            .fetch_update(Ordering::Release, Ordering::Acquire, |pos| {
                let a = alloc(self.slice, pos, layout)?;
                ptr = a.ptr;
                Some(a.new_pos)
            })
            .map_err(|_| AllocError)?;

        Ok(ptr)
    }

    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.allocate(layout)?;
        if !self.zeroed {
            unsafe { ptr.as_ptr().write_bytes(0, layout.size()) };
        }
        Ok(ptr)
    }
}

fn alloc(slice: NonNull<[u8]>, pos: usize, layout: NonZeroLayout) -> Option<SliceAlloc> {
    let base: *mut u8 = slice.as_ptr().cast();
    let ptr = unsafe { base.add(pos) };
    let align_offset = ptr.align_offset(layout.align());
    let total_size = align_offset.checked_add(layout.size())?;
    let remain = slice.len() - pos;

    if remain < total_size {
        return None;
    }

    let ptr = unsafe { ptr.add(align_offset) };
    let ptr = NonNull::new(ptr).unwrap();

    let new_pos = pos + total_size;

    Some(SliceAlloc { new_pos, ptr })
}

#[derive(Debug)]
struct SliceAlloc {
    new_pos: usize,
    ptr: NonNull<u8>,
}
