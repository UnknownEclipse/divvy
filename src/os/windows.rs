use core::{
    ptr::{self, NonNull},
    todo,
};
use std::io;

use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};
use windows_sys::Win32::System::Memory::{
    VirtualAlloc, VirtualFree, MEM_DECOMMIT, MEM_RELEASE, PAGE_PROTECTION_FLAGS,
    VIRTUAL_ALLOCATION_TYPE,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct VirtualAlloc {
    address: Option<NonNull<u8>>,
    allocation_type: VIRTUAL_ALLOCATION_TYPE,
    protect: PAGE_PROTECTION_FLAGS,
}

impl VirtualAlloc {
    pub fn virtual_alloc(&self, pages: usize) -> io::Result<NonNull<u8>> {
        let address = self.address.map(|p| p.as_ptr()).unwrap_or(ptr::null_mut());
        let size = pages
            .checked_mul(page_size())
            .ok_or(io::ErrorKind::InvalidInput)?;
        let ptr = unsafe { VirtualAlloc(address.cast(), size, self.allocation_type, self.protect) };

        if let Some(ptr) = NonNull::new(ptr) {
            Ok(ptr.cast())
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub unsafe fn virtual_free(&self, ptr: NonNull<u8>, pages: usize) -> io::Result<()> {
        let size = pages * page_size();
        let free_type = MEM_DECOMMIT | MEM_RELEASE;
        let ok = unsafe { VirtualFree(ptr.as_ptr().cast(), size, free_type) };

        if ok == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

unsafe impl Allocate for VirtualAlloc {
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let page_size = page_size();

        if page_size < layout.align() {
            todo!("huge alignment");
        }

        let pages = (layout.size() + page_size - 1) / page_size;
        self.virtual_alloc(pages).map_err(|_| AllocError)
    }
}

unsafe impl Deallocate for VirtualAlloc {
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        let page_size = page_size();

        if page_size < layout.align() {
            todo!("huge alignment");
        }

        let pages = (layout.size() + page_size - 1) / page_size;
        self.virtual_free(ptr, pages);
    }
}

pub fn page_size() -> usize {
    todo!()
}

pub const fn supports_deallocation() -> bool {
    true
}
