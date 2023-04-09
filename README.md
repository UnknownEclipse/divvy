# Divvy

Composable allocators for Rust projects using the unstable allocators API.

It's early days yet, so only a few basic primitives are fully implemented,
however with those out of the way the next step is higher-order allocators.
Pools, non-thread-safe slabs, lock-free slabs, arenas, etc.

The ultimate goal of this crate is to be able to easily compose allocators
together to build reliable, fast, and flexible allocators for any use case
without any unsafety.

## Available Primitives

### `Never`

The `Never` allocator does exactly what it sounds like; it never succeeds! This
can be useful for testing or as an equivalent to `!`/`Infallible` for generic
allocators.

Available on `no_std`.

```rust
use divvy::Never;

assert!(Box::try_new_in(0x42, Never).is_err());
```

### `Os`

The `Os` allocator acts as the thinnest possible wrapper over the operating
system's memory allocation facilities. On Unix this is `mmap(2)`, on Windows
`VirtualAlloc`, and so on.

Notably, this differs from the default `Global` allocator, which by default
calls libc's `malloc()`, which is a complex allocator built _on top_ of OS
facilities.

This is intended to be used as a backing allocator for more sophisticated
systems or for mapping huge regions of memory.

```rust
use divvy::Os;

// Allocate half a gigabyte directly from the os.
let mut arena = Vec::with_capacity_in(1 << 30, Os);
```

### `WrapAsGlobal`

`WrapAsGlobal` acts as a translation between the newer allocator api and the
existing `GlobalAlloc` api. Any allocator can be wrapped and then used as a
global allocator. Once the allocators api is farther along, this likely won't be
necessary.

Available on `no_std`.

```rust
struct MyAllocator { /* ... */ }

unsafe impl Allocator for MyAllocator { /* ... */ }

#[global_allocator]
static GLOBAL_ALLOC: WrapAsGlobal<MyAllocator> = WrapAsGlobal::new(MyAllocator::new());
```

## Planned

These interfaces are in progress but still not fully complete

### `Slab`

The `Slab` allocator is a highly space-efficient allocator for fixed size memory
blocks. There will be two versions available; sync and unsync. The sync
implementation uses a single-allocator, multiple deallocator atomic stack.

Attempting to allocate a memory block larger than a slab's set layout will fail,
and it is generally recommended not to underfill blocks to maintain lower memory
usage.

The sync slab implementation splits itself into a `local`, which can both
allocate and deallocate, and a `shared`, which may only deallocate. The `local`
may only be accessed from a single thread at a time, while the shared may be
freely accessed. This allows for a very efficient lock-free implementation.
Locals are typically held in a thread-local variable to allow for highly
efficient concurrent allocations, ala `mimalloc`.

Available on `no_std`

```rust
use divvy::{slab, Os};

// Create a slab that allocates memory blocks aligned and sized for u32 (or any other
// type with the same layout) backed by the `Os` allocator.
let layout = Layout::new::<u32>();

let Slab { local, shared } = SlabBuilder::new(layout)
    .limit(1 << 16) // Only allocate up to 16 kib
    .growth_policy(SlabGrowthPolicy::Pow2)
    .finish(Os)?;
```
