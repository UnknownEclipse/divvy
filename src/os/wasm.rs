use std::{arch::wasm32, ptr::NonNull};

use divvy_core::{AllocError, Allocate, NonZeroLayout};

#[derive(Debug, Default, Clone, Copy)]
pub struct Memory {
    _p: (),
}

impl Memory {
    #[inline]
    pub fn page_size(&self) -> usize {
        1024 * 64
    }

    #[inline]
    pub fn size(&self) -> usize {
        wasm32::memory_size() * self.page_size()
    }
}

unsafe impl Allocate for Memory {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.allocate_zeroed(layout)
    }

    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        if self.page_size() < layout.align() {
            todo!("huge align");
        }
        let pages = (layout.size() + self.page_size() - 1) / self.page_size();
        let page = wasm32::memory_grow::<0>(pages);
        if page == usize::MAX {
            Err(AllocError)
        } else {
            Ok(NonNull::new((page * self.page_size()) as *mut u8).unwrap())
        }
    }
}
