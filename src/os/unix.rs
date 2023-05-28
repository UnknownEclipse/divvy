use std::{
    io,
    num::NonZeroU32,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU32, Ordering},
};

use divvy_core::Deallocate;

use crate::{AllocError, Allocate, NonZeroLayout};

#[derive(Debug, Clone, Copy)]
pub struct Mmap {
    addr: Option<NonNull<u8>>,
    flags: i32,
    prot: i32,
}

impl Default for Mmap {
    fn default() -> Self {
        Self {
            addr: None,
            flags: libc::MAP_ANONYMOUS | libc::MAP_PRIVATE,
            prot: libc::PROT_WRITE | libc::PROT_READ,
        }
    }
}

impl Mmap {
    fn map(&self, size: usize) -> io::Result<NonNull<u8>> {
        let page_size = page_size();

        let size = (size + page_size - 1) / page_size * page_size;

        let ptr = unsafe { libc::mmap(ptr::null_mut(), size, self.prot, self.flags, -1, 0) };
        if ptr == libc::MAP_FAILED || ptr.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(NonNull::new(ptr).unwrap().cast())
        }
    }

    pub unsafe fn unmap(&self, ptr: NonNull<u8>, pages: usize) {
        unsafe { libc::munmap(ptr.as_ptr().cast(), pages * page_size()) };
    }
}

unsafe impl Allocate for Mmap {
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let page_size = page_size();
        if page_size < layout.align() {
            return Err(AllocError);
        }

        self.map(layout.size()).map_err(|_| AllocError)
    }
}

unsafe impl Deallocate for Mmap {
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        let page_size = page_size();
        let pages = (layout.size() + page_size - 1) / page_size;
        unsafe { self.unmap(ptr, pages) };
    }
}

pub const fn supports_deallocation() -> bool {
    true
}

#[inline]
fn page_size() -> usize {
    static PAGE_SIZE: AtomicU32 = AtomicU32::new(0);

    let mut page_shift = PAGE_SIZE.load(Ordering::Relaxed);
    if page_shift == 0 {
        page_shift = get_page_shift().get();
        PAGE_SIZE.store(page_shift, Ordering::Relaxed);
    }
    1 << page_shift
}

#[cold]
fn get_page_shift() -> NonZeroU32 {
    let page_size = unsafe { libc::sysconf(libc::_SC_PAGE_SIZE) };
    let page_size: u64 = page_size.try_into().expect("invalid page size");
    assert!(page_size.is_power_of_two(), "invalid page size");
    let shift = page_size.trailing_zeros();
    NonZeroU32::new(shift).expect("invalid page shift")
}
