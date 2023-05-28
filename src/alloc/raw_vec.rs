use core::slice;
use std::{
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

pub struct RawVec<T> {
    ptr: NonNull<T>,
    len: usize,
}

impl<T> RawVec<T> {
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: 0,
        }
    }
}

impl<T> Deref for RawVec<T> {
    type Target = [MaybeUninit<T>];

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { slice::from_raw_parts(self.ptr.as_ptr().cast(), self.len) }
    }
}

impl<T> DerefMut for RawVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr().cast(), self.len) }
    }
}
