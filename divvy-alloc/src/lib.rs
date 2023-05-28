use std::{alloc::Layout, ptr::NonNull};

use divvy_core::{AllocError, Allocate, NonZeroLayout};

mod boxed;
mod rc;
mod sync;

#[inline]
fn allocate_layout(layout: Layout, alloc: impl Allocate) -> Result<NonNull<u8>, AllocError> {
    if let Some(layout) = NonZeroLayout::new(layout) {
        alloc.allocate(layout)
    } else {
        let ptr = layout.align() as *mut u8;
        Ok(NonNull::new(ptr).unwrap())
    }
}
