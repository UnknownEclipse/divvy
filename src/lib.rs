// #![allow(clippy::missing_safety_doc)]
// #![cfg_attr(feature = "nightly", feature(allocator_api))]
// #![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(any(feature = "alloc", feature = "std"))]
// extern crate alloc;

// pub use divvy_core::*;

// pub use crate::{
//     arena::Arena,
//     global::{Global, WrapAsGlobal},
//     leak::Leak,
//     os::Os,
//     system::System,
// };

// mod allocation;
// mod arena;
// mod defaults;
// #[cfg(any(feature = "alloc", feature = "std"))]
// mod global;
// mod leak;
// mod never;
// #[cfg(feature = "nightly")]
// mod nightly;
// #[cfg(feature = "std")]
// mod os;
// mod slab;
// mod slice;
// mod sync_arena;
// #[cfg(feature = "std")]
// mod system;
