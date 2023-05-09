use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::Cell,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

use self::raw::SyncRawArena;
use crate::UnsafeBuf;

pub mod raw;

pub struct Arena<A> {
    backing: A,
    head: AtomicPtr<Link>,
}

impl<A> Arena<A> {}

#[repr(C)]
struct Link {
    next: Cell<Option<NonNull<Link>>>,
}

enum Policy {
    Fixed(Layout),
}
pub struct UnsyncArena<A> {
    backing: A,
    head: Cell<Option<NonNull<Link>>>,
}

unsafe impl<A> Allocator for UnsyncArena<A>
where
    A: Allocator,
{
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        todo!()
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        todo!()
    }
}

pub struct BufArena<B> {
    raw: SyncRawArena,
    buf: B,
}

impl<B> BufArena<B>
where
    B: UnsafeBuf,
{
    pub fn new(buf: B) -> Self {
        let ptr = buf.get();
        let raw = unsafe { SyncRawArena::new(ptr) };
        Self { raw, buf }
    }

    pub unsafe fn into_inner(self) -> B {
        self.buf
    }
}

unsafe impl<B> Allocator for BufArena<B>
where
    B: UnsafeBuf,
{
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.raw.allocate(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        self.raw.deallocate(ptr, layout);
    }
}
