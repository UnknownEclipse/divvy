// pub struct Arena<A> {}

use core::{
    alloc::Layout,
    cell::{Cell, UnsafeCell},
    cmp, mem,
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};

#[derive(Debug, Default)]
pub struct Arena<A>
where
    A: Deallocate,
{
    raw: UnsafeCell<RawArena>,
    alloc: A,
}

impl<A> Arena<A>
where
    A: Deallocate,
{
    #[inline]
    pub const fn new(alloc: A) -> Self {
        Self {
            raw: UnsafeCell::new(RawArena::new()),
            alloc,
        }
    }
}

unsafe impl<A> Allocate for Arena<A>
where
    A: Allocate + Deallocate,
{
    #[inline]
    fn allocate(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        self.allocate_zeroed(layout)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: NonZeroLayout) -> Result<NonNull<u8>, AllocError> {
        let raw = unsafe { &mut *self.raw.get() };

        if let Some(p) = raw.allocate_in_current_chunk(layout) {
            Ok(p)
        } else {
            raw.allocate_slow(layout, &self.alloc)
        }
    }
}

impl<A> Drop for Arena<A>
where
    A: Deallocate,
{
    fn drop(&mut self) {
        unsafe { self.raw.get_mut().clear(&self.alloc) };
    }
}

#[derive(Debug, Default)]
struct RawArena {
    current: Option<NonNull<Footer>>,
}

impl RawArena {
    pub const fn new() -> Self {
        Self { current: None }
    }

    #[inline]
    pub fn allocate_in_current_chunk(&mut self, layout: NonZeroLayout) -> Option<NonNull<u8>> {
        self.footer().and_then(|f| f.allocate_in(layout))
    }

    #[cold]
    #[inline(never)]
    pub fn allocate_slow(
        &mut self,
        layout: NonZeroLayout,
        allocate: &dyn Allocate,
    ) -> Result<NonNull<u8>, AllocError> {
        if let Some(ptr) = self.allocate_in_current_chunk(layout) {
            return Ok(ptr);
        }
        self.allocate_chunk(layout, allocate)?;
        self.allocate_in_current_chunk(layout).ok_or(AllocError)
    }

    fn allocate_chunk(
        &mut self,
        min_layout: NonZeroLayout,
        allocate: &dyn Allocate,
    ) -> Result<(), AllocError> {
        let min_size = min_layout.size() + mem::size_of::<Footer>();
        let size = self
            .footer()
            .map(|f| f.next_chunk_size())
            .unwrap_or(1 << 10);

        let size = cmp::max(min_size, size);
        let align = cmp::min(min_layout.align(), mem::align_of::<Footer>());

        let layout = Layout::from_size_align(size, align).map_err(|_| AllocError)?;
        let (layout, footer_offset) = layout
            .extend(Layout::new::<Footer>())
            .map_err(|_| AllocError)?;

        let layout = NonZeroLayout::new(layout).unwrap();

        let ptr = allocate.allocate_zeroed(layout).unwrap();
        let footer = unsafe { ptr.as_ptr().add(footer_offset).cast() };
        unsafe {
            ptr::write(
                footer,
                Footer {
                    layout,
                    next: self.current,
                    pos: Cell::new(ptr),
                    ptr,
                },
            );
        }
        let footer = NonNull::new(footer);
        self.current = footer;
        Ok(())
    }

    #[inline]
    fn footer(&self) -> Option<&Footer> {
        self.current.map(|p| unsafe { p.as_ref() })
    }

    pub unsafe fn clear(&mut self, dealloc: &dyn Deallocate) {
        let mut chunk = self.current;
        while let Some(footer) = chunk {
            chunk = unsafe { footer.as_ref().next };
            unsafe { deallocate_chunk(footer, dealloc) };
        }
    }
}

#[derive(Debug)]
struct Footer {
    layout: NonZeroLayout,
    next: Option<NonNull<Footer>>,
    ptr: NonNull<u8>,
    pos: Cell<NonNull<u8>>,
}

impl Footer {
    fn next_chunk_size(&self) -> usize {
        self.size() * 2
    }

    #[inline]
    fn size(&self) -> usize {
        self.layout.size() - mem::size_of::<Footer>()
    }

    #[inline]
    #[allow(dead_code)]
    fn allocate_in_safe(&self, layout: NonZeroLayout) -> Option<NonNull<u8>> {
        let ptr = self.pos.get().as_ptr();
        let end_ptr = self as *const Self as *const u8;
        let remaining = (end_ptr as usize) - (ptr as usize);

        let align_offset = ptr.align_offset(layout.align());
        let total_size = align_offset + layout.size();

        if remaining < total_size {
            return None;
        }

        let ptr = unsafe { ptr.add(align_offset) };

        let new_pos = unsafe { ptr.add(layout.size()) };
        self.pos.set(NonNull::new(new_pos).unwrap());

        NonNull::new(ptr)
    }

    #[inline]
    fn allocate_in(&self, layout: NonZeroLayout) -> Option<NonNull<u8>> {
        self.allocate_in_ub(layout)
    }

    /// This implementation is roughly 30% faster, but hinges on uncertain pointer
    /// provenance rules.
    #[inline]
    fn allocate_in_ub(&self, layout: NonZeroLayout) -> Option<NonNull<u8>> {
        let ptr = self.pos.get().as_ptr();
        let end_ptr = self as *const Self as *const u8;
        // let remaining = (end_ptr as usize) - (ptr as usize);

        let align_offset = ptr.align_offset(layout.align());
        // let total_size = align_offset + layout.size();

        let aligned_ptr = unsafe { ptr.add(align_offset) };
        let new_pos = unsafe { ptr.add(layout.size()) };

        if end_ptr <= new_pos {
            return None;
        }

        // let ptr = unsafe { ptr.add(align_offset) };

        // let new_pos = unsafe { ptr.add(layout.size()) };
        self.pos.set(NonNull::new(new_pos).unwrap());

        NonNull::new(aligned_ptr)
    }
}

unsafe fn deallocate_chunk(chunk: NonNull<Footer>, deallocate: &dyn Deallocate) {
    let footer = unsafe { chunk.as_ref() };
    let layout = footer.layout;
    let ptr = footer.ptr;

    unsafe { deallocate.deallocate(ptr, layout) };
}
