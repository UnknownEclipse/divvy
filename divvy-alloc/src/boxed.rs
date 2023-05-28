use std::{
    alloc::Layout,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};

use crate::allocate_layout;

#[non_exhaustive]
#[derive(Debug)]
pub enum TryNewError<T> {
    AllocError { value: T },
}

#[derive(Debug)]
pub struct Box<T, A>
where
    A: Deallocate,
{
    ptr: NonNull<T>,
    alloc: A,
}

impl<T, A> Box<T, A>
where
    A: Allocate + Deallocate,
{
    #[inline]
    pub fn try_new_uninit_in(alloc: A) -> Result<Box<MaybeUninit<T>, A>, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = allocate_layout(layout, &alloc)?;

        Ok(Box {
            ptr: ptr.cast(),
            alloc,
        })
    }

    #[inline]
    pub fn try_new_in(value: T, alloc: A) -> Result<Box<MaybeUninit<T>, A>, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = allocate_layout(layout, &alloc)?;

        Ok(Box {
            ptr: ptr.cast(),
            alloc,
        })
    }
}

impl<T, A> Box<T, A>
where
    A: Deallocate,
{
    #[inline]
    pub unsafe fn from_raw_with_alloc(raw: *mut T, alloc: A) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(raw) };
        Self { ptr, alloc }
    }

    #[inline]
    pub fn into_raw_with_alloc(this: Self) -> (*mut T, A) {
        let this = ManuallyDrop::new(this);
        let ptr = this.ptr.as_ptr();
        let alloc = unsafe { ptr::read(&this.alloc) };
        (ptr, alloc)
    }
}

impl<T, A> Box<MaybeUninit<T>, A>
where
    A: Deallocate,
{
    pub fn write(mut this: Self, value: T) -> Box<T, A> {
        this.deref_mut().write(value);
        let (ptr, alloc) = Box::into_raw_with_alloc(this);
        unsafe { Box::from_raw_with_alloc(ptr.cast(), alloc) }
    }
}

impl<T, A> Box<T, A> where A: Deallocate {}

impl<T, A> Deref for Box<T, A>
where
    A: Deallocate,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A> DerefMut for Box<T, A>
where
    A: Deallocate,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T, A> Drop for Box<T, A>
where
    A: Deallocate,
{
    #[inline]
    fn drop(&mut self) {
        unsafe { self.ptr.as_ptr().drop_in_place() };
        let layout = Layout::new::<T>();
        if let Some(layout) = NonZeroLayout::new(layout) {
            let ptr = self.ptr.cast();
            unsafe { self.alloc.deallocate(ptr, layout) };
        }
    }
}

fn alloc<T, F, D>(f: F) -> Result<Box<MaybeUninit<T>, D>, AllocError>
where
    D: Deallocate,
    F: FnOnce() -> Result<(NonNull<u8>, D), AllocError>,
{
    f().map(|(ptr, alloc)| Box {
        ptr: ptr.cast(),
        alloc,
    })
}
