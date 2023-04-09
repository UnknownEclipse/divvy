use core::{
    alloc::{AllocError, Allocator, Layout},
    cell::Cell,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub struct SyncRawArena {
    pos: AtomicUsize,
    arena: NonNull<[u8]>,
}

impl SyncRawArena {
    /// Create a new arena.
    ///
    /// # Safety
    /// The provided arena pointer must remain valid for the lifetime of this arena.
    #[inline]
    pub const unsafe fn new(arena: NonNull<[u8]>) -> Self {
        Self {
            pos: AtomicUsize::new(0),
            arena,
        }
    }

    /// Clears this arena
    ///
    /// # Safety
    /// This method must not be called while any allocations are still in use. Doing
    /// so *will* cause a use-after-free;
    #[inline]
    pub unsafe fn clear(&self) {
        self.pos.store(0, Ordering::Release);
    }

    #[inline]
    pub fn data(&self) -> NonNull<[u8]> {
        self.arena
    }
}

unsafe impl Allocator for SyncRawArena {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let mut result = NonNull::from(&[][..]);

        let ok = self
            .pos
            .fetch_update(Ordering::Acquire, Ordering::Relaxed, |pos| {
                let bump = arena_alloc(self.arena, pos, layout).ok()?;
                let Bump { ptr, new_pos } = bump;
                result = ptr;
                Some(new_pos)
            })
            .is_ok();

        if ok {
            Ok(result)
        } else {
            Err(AllocError)
        }
    }

    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

#[derive(Debug)]
pub struct UnsyncRawArena {
    pos: Cell<usize>,
    arena: NonNull<[u8]>,
}

impl UnsyncRawArena {
    /// Create a new arena.
    ///
    /// # Safety
    /// The provided arena pointer must remain valid for the lifetime of this arena.
    #[inline]
    pub const unsafe fn new(arena: NonNull<[u8]>) -> Self {
        Self {
            pos: Cell::new(0),
            arena,
        }
    }

    /// Clears this arena
    ///
    /// # Safety
    /// This method must not be called while any allocations are still in use. Doing
    /// so *will* cause a use-after-free;
    #[inline]
    pub unsafe fn clear(&self) {
        self.pos.set(0);
    }

    #[inline]
    pub fn data(&self) -> NonNull<[u8]> {
        self.arena
    }
}

unsafe impl Allocator for UnsyncRawArena {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let pos = self.pos.get();
        let bump = arena_alloc(self.arena, pos, layout)?;
        let Bump { ptr, new_pos } = bump;
        self.pos.set(new_pos);
        Ok(ptr)
    }

    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

#[derive(Debug)]
struct Bump {
    ptr: NonNull<[u8]>,
    new_pos: usize,
}

/// The actual arena allocation function. This needs to take special care to ensure
/// correctness, so we try to use indexes as much as possible.
fn arena_alloc(arena: NonNull<[u8]>, pos: usize, layout: Layout) -> Result<Bump, AllocError> {
    let align = layout.align();
    let arena_start = arena.as_ptr().cast::<u8>();

    let align_offset = arena_start.wrapping_add(pos).align_offset(align);

    let pos = pos.checked_add(align_offset).ok_or(AllocError)?;

    arena_alloc_aligned(arena, pos, layout)
}

/// The actual arena allocation function. This needs to take special care to ensure
/// correctness, so we try to use indexes as much as possible.
///
/// This variant assumes the pointer is already aligned
fn arena_alloc_aligned(
    arena: NonNull<[u8]>,
    pos: usize,
    layout: Layout,
) -> Result<Bump, AllocError> {
    let arena_start = arena.as_ptr().cast::<u8>();

    let start = pos;
    let end = start.checked_add(layout.size()).ok_or(AllocError)?;

    if arena.len() < end {
        return Err(AllocError);
    }

    let ptr = unsafe { arena_start.add(start) };
    let ptr = NonNull::new(ptr).expect("ptr should not be null");
    let len = layout.size();
    let ptr = NonNull::slice_from_raw_parts(ptr, len);

    Ok(Bump { ptr, new_pos: end })
}
