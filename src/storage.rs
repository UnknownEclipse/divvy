use alloc::alloc::handle_alloc_error;
use core::{
    alloc::{AllocError, Allocator, Layout},
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

pub unsafe trait Storage {
    type Handle: Copy + Eq;

    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<[u8]>;

    fn allocate(&self, layout: Layout) -> Result<Self::Handle, AllocError>;

    unsafe fn deallocate(&self, handle: Self::Handle, layout: Layout);
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AllocatorStorage<A>(A);

unsafe impl<A> Storage for AllocatorStorage<A>
where
    A: Allocator,
{
    type Handle = NonNull<[u8]>;

    #[inline]
    unsafe fn resolve(&self, handle: Self::Handle) -> NonNull<[u8]> {
        handle
    }

    #[inline]
    fn allocate(&self, layout: Layout) -> Result<Self::Handle, AllocError> {
        self.0.allocate(layout)
    }

    #[inline]
    unsafe fn deallocate(&self, handle: Self::Handle, layout: Layout) {
        self.0.deallocate(handle.as_non_null_ptr(), layout)
    }
}

pub struct Box<T, S>
where
    S: Storage,
{
    storage: S,
    handle: S::Handle,
    _p: PhantomData<T>,
}

impl<T, S> Box<T, S>
where
    S: Storage,
{
    pub fn new_in(value: T, storage: S) -> Self {
        match Self::try_new_in(value, storage) {
            Ok(v) => v,
            Err(_) => handle_alloc_error(Layout::new::<T>()),
        }
    }

    pub fn try_new_in(value: T, storage: S) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let handle = storage.allocate(layout)?;

        unsafe {
            let ptr = storage.resolve(handle);
            ptr::write(ptr.as_ptr().cast(), value);
        }

        Ok(Self {
            storage,
            handle,
            _p: PhantomData,
        })
    }

    pub fn into_raw_with_storage(this: Box<T, S>) -> (S::Handle, S) {
        let handle = this.handle;
        let storage = unsafe { ptr::read(&this.storage) };
        mem::forget(this);
        (handle, storage)
    }

    pub fn into_inner(this: Box<T, S>) -> T {
        let (handle, storage) = Box::into_raw_with_storage(this);

        let ptr = unsafe { storage.resolve(handle) };
        let value = unsafe { ptr::read(ptr.as_ptr().cast()) };
        let layout = Layout::new::<T>();

        unsafe { storage.deallocate(handle, layout) };

        value
    }

    fn as_ptr(this: &Box<T, S>) -> *mut T {
        unsafe { this.storage.resolve(this.handle).as_ptr().cast() }
    }
}

impl<T, S> Deref for Box<T, S>
where
    S: Storage,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Box::as_ptr(self) }
    }
}

impl<T, S> DerefMut for Box<T, S>
where
    S: Storage,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *Box::as_ptr(self) }
    }
}

impl<T, S> Drop for Box<T, S>
where
    S: Storage,
{
    fn drop(&mut self) {
        let ptr = Box::as_ptr(self);
        let layout = Layout::new::<T>();

        unsafe {
            ptr::drop_in_place(ptr);
            self.storage.deallocate(self.handle, layout)
        }
    }
}
