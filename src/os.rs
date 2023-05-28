use std::ptr::NonNull;

use cfg_if::cfg_if;
use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};

cfg_if! {
    if #[cfg(unix)] {
        pub mod unix;
        use unix::Mmap as Imp;
        use unix as imp;
    } else if #[cfg(target_family = "wasm")] {
        mod wasm;
    } else if #[cfg(windows)] {
        pub mod windows;
        use windows as imp;
        use windows::VirtualAlloc as Imp;
    } else {
        compile_error!("os allocator not supported on current platform");
    }
}

/// A low-level interface to the current platform's virtual memory functions, such as
/// mmap on unix and VirtualAlloc on windows. On its own, this will have terrible
/// performance, especially for small allocations. It should instead be used as the
/// backing allocator for more advanced systems.
///
/// Note that not all platforms (notably WASM) support deallocation of pages. To deallocate OS
/// memory, use either the `.deallocator()` function or the direct interfaces
/// (Mmap, VirtualAlloc, MemoryGrow, etc.).
#[derive(Debug, Default, Clone, Copy)]
pub struct Os;

impl Os {
    #[inline]
    pub const fn deallocator(&self) -> Option<OsDeallocator> {
        if imp::supports_deallocation() {
            Some(OsDeallocator(()))
        } else {
            None
        }
    }
}

unsafe impl Allocate for Os {
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        // println!("os.alloc: {:?}", layout);
        Imp::default().allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        Imp::default().allocate_zeroed(layout)
    }

    #[inline]
    unsafe fn try_grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        unsafe { Imp::default().try_grow(ptr, old_layout, new_layout) }
    }

    #[inline]
    unsafe fn try_grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: NonZeroLayout,
        new_layout: NonZeroLayout,
    ) -> Option<NonNull<u8>> {
        unsafe { Imp::default().try_grow_zeroed(ptr, old_layout, new_layout) }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OsDeallocator(());

unsafe impl Deallocate for OsDeallocator {
    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: NonZeroLayout) {
        unsafe { Imp::default().deallocate(ptr, layout) };
    }
}
