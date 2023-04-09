use std::{
    alloc::{AllocError, Layout},
    io,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU32, Ordering},
};

use libc::{
    mmap, munmap, sysconf, MAP_ANONYMOUS, MAP_FAILED, MAP_FIXED, MAP_PRIVATE, PROT_READ,
    PROT_WRITE, _SC_PAGE_SIZE,
};

pub fn os_alloc(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    let page_size = page_size();

    if layout.align() <= page_size {
        let pages = needed_pages(layout.size());
        map_pages(pages, None, false, false)
    } else {
        os_alloc_huge_align(layout)
    }
}

pub fn os_alloc_zeroed(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    // mmap gives out zeroed pages
    os_alloc(layout)
}

pub unsafe fn os_dealloc(ptr: NonNull<u8>, layout: Layout) {
    let pages = needed_pages(layout.size());
    unsafe { unmap_pages(ptr, pages) };
}

pub unsafe fn os_grow(
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    let pages = needed_pages(new_layout.size());
    let new = map_pages(pages, Some(ptr), false, false)?;

    if new.cast() != ptr {
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new.as_ptr().cast(), old_layout.size());
            os_dealloc(ptr, old_layout);
        }
    }

    Ok(new)
}

pub unsafe fn os_grow_zeroed(
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    os_grow(ptr, old_layout, new_layout)
}

pub unsafe fn os_shrink(
    ptr: NonNull<u8>,
    old_layout: Layout,
    new_layout: Layout,
) -> Result<NonNull<[u8]>, AllocError> {
    let old_pages = needed_pages(old_layout.size());
    let new_pages = needed_pages(new_layout.size());

    let to_unmap = old_pages.saturating_sub(new_pages);
    if to_unmap != 0 {
        let unmap_base = ptr.as_ptr().add(new_pages * page_size());
        let unmap_base = NonNull::new(unmap_base).expect("ptr is never null");
        unmap_pages(unmap_base, to_unmap);
    }
    let ptr = NonNull::slice_from_raw_parts(ptr, new_pages * page_size());
    Ok(ptr)
}

fn os_alloc_huge_align(layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    debug_assert!(page_size() < layout.align());

    let overaligned_layout =
        Layout::from_size_align(layout.size() + layout.align(), layout.align())
            .map_err(|_| AllocError)?;

    let pages = needed_pages(overaligned_layout.size());
    let overaligned = map_pages(pages, None, false, true)?;
    let overaligned_ptr: *mut u8 = overaligned.as_ptr().cast();
    let align_offset = overaligned_ptr.align_offset(layout.align());

    if align_offset != 0 {
        let page_size = page_size();
        assert_eq!(align_offset % page_size, 0);
        let pages = align_offset / page_size;
        unsafe { unmap_pages(overaligned.cast(), pages) };
    }

    let ptr = unsafe { overaligned_ptr.add(align_offset) };
    let ptr = NonNull::new(ptr).expect("ptr is non-null");

    let len = overaligned
        .len()
        .checked_sub(align_offset)
        .expect("unexpected overflow");

    let ptr = NonNull::slice_from_raw_parts(ptr, len);
    unsafe { update_page_prot(ptr) };
    Ok(ptr)
}

fn needed_pages(size: usize) -> usize {
    let page_size = page_size();
    (size + page_size - 1) / page_size
}

fn map_pages(
    pages: usize,
    addr: Option<NonNull<u8>>,
    fixed: bool,
    weak: bool,
) -> Result<NonNull<[u8]>, AllocError> {
    let addr = addr.map(|p| p.as_ptr()).unwrap_or(ptr::null_mut()).cast();

    let len = pages.checked_mul(page_size()).ok_or(AllocError)?;
    let prot = if weak { 0 } else { PROT_WRITE | PROT_READ };
    let mut flags = MAP_ANONYMOUS | MAP_PRIVATE;
    if fixed {
        flags |= MAP_FIXED;
    }
    let fd = -1;
    let offset = 0;

    let res = unsafe { mmap(addr, len, prot, flags, fd, offset) };

    if res == MAP_FAILED {
        Err(AllocError)
    } else {
        let ptr = NonNull::new(res).ok_or(AllocError)?;
        let ptr = NonNull::slice_from_raw_parts(ptr.cast(), len);
        Ok(ptr)
    }
}

fn page_size() -> usize {
    static PAGE_SHIFT: AtomicU32 = AtomicU32::new(0);

    // Safety: Relaxed ordering is fine. page_shift will only ever be 0 or
    //         log2(page_size).
    let mut page_shift = PAGE_SHIFT.load(Ordering::Relaxed);
    if page_shift == 0 {
        page_shift = get_page_shift();
        PAGE_SHIFT.store(page_shift, Ordering::Relaxed);
    }

    1 << page_shift
}

#[cold]
fn get_page_shift() -> u32 {
    let result = unsafe { sysconf(_SC_PAGE_SIZE) };

    // The OS gave us a bad page size. If that happens, things are royally boned
    // anyway so panic.
    let page_size: usize = result.try_into().expect("invalid page size");
    assert!(page_size.is_power_of_two(), "invalid page size");

    // log2 to get shift value
    page_size.trailing_zeros()
}

unsafe fn unmap_pages(ptr: NonNull<u8>, pages: usize) {
    // We compute the amount of pages by the size, which should never overflow but this
    // won't hurt performance.
    let len = pages.checked_mul(page_size()).expect("unexpected overflow");

    let rc = unsafe {
        // Safety: The user must pass in a valid pointer
        // Note: In truly broken cases, the OS will be able to catch it, however that
        //       will only happen when things are fantastically screwy.
        munmap(ptr.as_ptr().cast(), len)
    };

    if rc != 0 {
        // Someone passed a *really* bogus pointer to Os.deallocate(). For now, just
        // panic. (If this is happening, the program is already broken beyond repair)
        munmap_failed(ptr);
    }
}

#[cold]
fn munmap_failed(ptr: NonNull<u8>) -> ! {
    let err = io::Error::last_os_error();
    panic!("munmap failed to deallocate {:p}: {}", ptr, err);
}

unsafe fn update_page_prot(ptr: NonNull<[u8]>) {
    unsafe { libc::mprotect(ptr.as_ptr().cast(), ptr.len(), PROT_READ | PROT_WRITE) };
}
