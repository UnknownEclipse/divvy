use core::{
    cell::{Cell, UnsafeCell},
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

struct Local {}

struct Shared {}

impl Shared {
    fn alloc(&self) {}
}

const PAGE_SIZE: usize = 1 << 16;
const PAGE_CAPACITY: usize = PAGE_SIZE - mem::size_of::<usize>() * 7;

#[repr(C, align(65536))]
#[derive(Debug)]
struct Page {
    free: Cell<Option<NonNull<Block>>>,
    local_free: Cell<Option<NonNull<Block>>>,
    shared_free: AtomicPtr<Block>,
    used: usize,
    next: Cell<Option<NonNull<Page>>>,
    prev: Cell<Option<NonNull<Page>>>,
    block_size: usize,
    buffer: UnsafeCell<[MaybeUninit<u8>; PAGE_CAPACITY]>,
}

impl Page {
    unsafe fn alloc(&self) -> Option<NonNull<[u8]>> {
        let mut head = self.free.get();

        if head.is_none() {
            self.take_remote_frees();
            head = self.free.get();
        }
        let block = head?;
        let new_head = unsafe { block.as_ref().next.get() };
        self.free.set(new_head);

        Some(NonNull::slice_from_raw_parts(block.cast(), self.block_size))
    }

    #[inline]
    unsafe fn maybe_compact_free(&self) {
        if self.free.get().is_none() {
            self.take_remote_frees();
        }
    }

    #[cold]
    unsafe fn take_remote_frees(&self) {
        let head = self.shared_free.swap(ptr::null_mut(), Ordering::Acquire);
        debug_assert!(self.free.get().is_none());
        self.free.set(NonNull::new(head));
    }

    unsafe fn free(&self, block: NonNull<Block>) {
        todo!()
    }
}

#[repr(transparent)]
#[derive(Debug, Default)]
struct Block {
    next: Cell<Option<NonNull<Block>>>,
}
