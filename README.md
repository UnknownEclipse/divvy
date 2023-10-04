# Divvy

Composable allocators for Rust projects.

It's early days yet, so only a few basic primitives are fully implemented,
however with those out of the way the next step is higher-order allocators.
Pools, non-thread-safe slabs, lock-free slabs, arenas, etc.

The ultimate goal of this crate is to be able to easily compose allocators
together to build reliable, fast, and flexible allocators for any use case
without any unsafety.
