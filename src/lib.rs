#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub use divvy_core::*;

#[cfg(feature = "alloc")]
pub use crate::global::{Global, WrapAsGlobal};
pub use crate::{fixed_slice::FixedSlice, never::Never};

mod fixed_slice;
#[cfg(feature = "alloc")]
mod global;
mod never;

#[inline]
unsafe fn sub_ptr<T>(left: *const T, right: *const T) -> usize {
    (left as usize) - (right as usize)
}
