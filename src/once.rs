use core::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
    sync::atomic::{AtomicBool, Ordering},
};

/// Enforces that exactly one call to `.allocate()` may be performed. Any further
/// allocations will result in a panic. This can be useful in resource constrained
/// systems to enforce that no allocations occur without explicit request.
#[derive(Debug, Default)]
pub struct Once<A> {
    alloc: A,
    tripped: AtomicBool,
}

impl<A> Once<A> {
    pub const fn new(alloc: A) -> Self {
        Self {
            alloc,
            tripped: AtomicBool::new(false),
        }
    }

    pub fn into_inner(self) -> A {
        self.alloc
    }
}

unsafe impl<A> Allocator for Once<A>
where
    A: Allocator,
{
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let was_tripped = self.tripped.swap(true, Ordering::Relaxed);
        assert!(!was_tripped, "attempted to allocate more than once");
        self.alloc.allocate(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let was_tripped = self.tripped.swap(true, Ordering::Relaxed);
        assert!(!was_tripped, "attempted to allocate more than once");
        self.alloc.allocate_zeroed(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        assert!(
            self.tripped.load(Ordering::Relaxed),
            "attempted to deallocate invalid ptr"
        );
        self.alloc.deallocate(ptr, layout);
    }
}
