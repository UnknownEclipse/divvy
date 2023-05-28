use core::ptr::NonNull;

use divvy_core::{Deallocate, NonZeroLayout};

#[derive(Debug)]
pub struct Allocation<D>
where
    D: Deallocate,
{
    ptr: NonNull<u8>,
    layout: NonZeroLayout,
    dealloc: D,
}

impl<D> Allocation<D> where D: Deallocate {}
