use core::{
    alloc::{Allocator, GlobalAlloc, Layout},
    cmp::Ordering,
    ptr::{self, NonNull},
};

/// Wrap an allocator so that it may used as a global allocator. See [GlobalAlloc] for
/// more information. This hopefully won't be needed in the future once the allocator
/// api gets closer to being stable.
///
/// # Examples
///
/// Ban all allocations, *Mwahahaha*!
/// ```rust
/// # use std::panic;
///
/// use divvy::{Never, WrapAsGlobal};
///
/// #[global_allocator]
/// static GLOBAL_ALLOCATOR: WrapAsGlobal<Never> = WrapAsGlobal::new(Never);
///
/// let result = panic::catch_unwind(|| {
///     let s = String::from("Whoops!");
///     println!("Hmm... why didn't things break? {}", s);
/// });
///
/// // Panicked!
/// assert!(result.is_err());
/// ```
#[derive(Debug, Default)]
pub struct WrapAsGlobal<A> {
    alloc: A,
}

impl<A> WrapAsGlobal<A> {
    #[inline]
    pub const fn new(alloc: A) -> Self {
        Self { alloc }
    }

    #[inline]
    pub fn into_inner(self) -> A {
        self.alloc
    }

    #[inline]
    pub fn get(&self) -> &A {
        &self.alloc
    }

    #[inline]
    pub fn get_mut(&mut self) -> &mut A {
        &mut self.alloc
    }
}

unsafe impl<A> GlobalAlloc for WrapAsGlobal<A>
where
    A: Allocator,
{
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        match self.alloc.allocate(layout) {
            Ok(ptr) => ptr.as_ptr().cast(),
            Err(_) => ptr::null_mut(),
        }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).expect("ptr passed to dealloc was null");
        unsafe { self.alloc.deallocate(ptr, layout) };
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        match self.alloc.allocate_zeroed(layout) {
            Ok(ptr) => ptr.as_ptr().cast(),
            Err(_) => ptr::null_mut(),
        }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if layout.size() == new_size {
            return ptr;
        }

        let old_layout = layout;
        let new_layout = match make_new_layout(layout, new_size) {
            Some(v) => v,
            None => return ptr::null_mut(),
        };
        let ptr = NonNull::new(ptr).expect("ptr passed to realloc was null");

        let result = match layout.size().cmp(&new_size) {
            Ordering::Less => unsafe { self.alloc.shrink(ptr, old_layout, new_layout) },
            Ordering::Greater => unsafe { self.alloc.grow(ptr, old_layout, new_layout) },
            Ordering::Equal => {
                // We already checked for equality and returned early
                unreachable!()
            }
        };

        match result {
            Ok(ptr) => ptr.as_ptr().cast(),
            Err(_) => ptr::null_mut(),
        }
    }
}

fn make_new_layout(layout: Layout, new_size: usize) -> Option<Layout> {
    Layout::from_size_align(new_size, layout.align()).ok()
}
