#![allow(clippy::missing_safety_doc)]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "alloc", feature = "std"))]
extern crate alloc;

pub use divvy_core::*;

pub use crate::{
    arena::Arena,
    leak::Leak,
    slice::{Slice, SyncSlice},
};

mod arena;
#[cfg(any(feature = "alloc", feature = "std"))]
mod global;
#[cfg(any(feature = "alloc", feature = "std"))]
pub use global::{Global, WrapAsGlobal};
mod leak;
mod never;
mod slice;
