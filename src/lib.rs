#![no_std]

pub use divvy_core::*;

pub use crate::{fixed_slice::FixedSlice, global::Global, never::Never};

mod fixed_slice;
mod global;
mod never;

#[inline]
unsafe fn sub_ptr<T>(left: *const T, right: *const T) -> usize {
    (left as usize) - (right as usize)
}
