#![cfg_attr(not(feature = "std"), no_std)]
#![feature(alloc_layout_extra, allocator_api, sync_unsafe_cell)]

extern crate alloc;

pub use crate::{never::Never, os::Os, wrap_as_global::WrapAsGlobal};

pub mod arena;
mod never;
#[cfg(feature = "std")]
mod os;
mod slab;
mod wrap_as_global;
