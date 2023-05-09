#![cfg_attr(not(feature = "std"), no_std)]
#![feature(alloc_layout_extra, allocator_api, slice_ptr_get, sync_unsafe_cell)]

extern crate alloc;

use core::{alloc::Allocator, cell::UnsafeCell, mem::MaybeUninit, pin::Pin, ptr::NonNull};

pub use crate::{never::Never, os::Os, wrap_as_global::WrapAsGlobal};

pub mod arena;
mod never;
mod once;
#[cfg(feature = "std")]
mod os;
mod pool;
mod slab;
mod storage;
mod wrap_as_global;

/// An allocator that works over a fixed region of memory only.
pub unsafe trait FixedAllocator: Allocator {
    unsafe fn with_buf(buf: NonNull<[u8]>) -> Self;
}

pub unsafe trait UnsafeBuf {
    fn get(&self) -> NonNull<[u8]>;
}

unsafe impl<A> UnsafeBuf for Pin<Box<UnsafeCell<[MaybeUninit<u8>]>, A>>
where
    A: Allocator,
{
    fn get(&self) -> NonNull<[u8]> {
        let ptr = UnsafeCell::get(self) as *mut [u8];
        NonNull::new(ptr).unwrap()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BorrowedBuf<'a>(&'a UnsafeCell<[MaybeUninit<u8>]>);

impl<'a> BorrowedBuf<'a> {
    pub fn from_slice(slice: &'a mut [u8]) -> Self {
        unsafe { Self::from_ptr(slice) }
    }

    pub fn from_uninit_slice(slice: &'a mut [MaybeUninit<u8>]) -> Self {
        let ptr = slice as *mut [_] as *mut [_];
        unsafe { Self::from_ptr(ptr) }
    }

    unsafe fn from_ptr(ptr: *mut [u8]) -> Self {
        let ptr = ptr as *const UnsafeCell<[MaybeUninit<u8>]>;
        Self(&*ptr)
    }
}

unsafe impl<'a> UnsafeBuf for BorrowedBuf<'a> {
    fn get(&self) -> NonNull<[u8]> {
        let ptr = self.0.get() as *mut [u8];
        NonNull::new(ptr).unwrap()
    }
}

unsafe impl UnsafeBuf for NonNull<[u8]> {
    fn get(&self) -> NonNull<[u8]> {
        *self
    }
}
