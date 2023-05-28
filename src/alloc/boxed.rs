use std::{
    alloc::{handle_alloc_error, Layout},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocate, Deallocate, NonZeroLayout};

use crate::Global;

pub struct Box<T: ?Sized, A>
where
    A: Deallocate,
{
    alloc: A,
    ptr: NonNull<T>,
    _p: PhantomData<T>,
}

#[derive(Debug)]
pub struct TryNewError<T>(pub T);

impl<T, A> Box<T, A>
where
    A: Allocate + Deallocate,
{
    pub fn try_new_uninit_in(alloc: A) -> Result<Box<MaybeUninit<T>, A>, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = match NonZeroLayout::new(layout) {
            Some(layout) => alloc.allocate(layout)?,
            None => NonNull::new(layout.align() as *mut u8).unwrap(),
        };

        let ptr = ptr.cast();

        Ok(Box {
            alloc,
            ptr,
            _p: PhantomData,
        })
    }

    pub fn try_new_in(value: T, alloc: A) -> Result<Box<T, A>, TryNewError<T>> {
        match Box::try_new_uninit_in(alloc) {
            Ok(b) => Ok(Box::write(b, value)),
            Err(_) => Err(TryNewError(value)),
        }
    }

    pub fn new_in(value: T, alloc: A) -> Box<T, A> {
        match Box::try_new_in(value, alloc) {
            Ok(b) => b,
            Err(_) => handle_alloc_error(Layout::new::<T>()),
        }
    }
}

impl<T> Box<T, Global> {
    pub fn new(value: T) -> Box<T, Global> {
        Box::new_in(value, Global)
    }
}

impl<T, A> Box<T, A>
where
    A: Deallocate,
{
    pub fn as_ptr(&self) -> *const T {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr.as_ptr()
    }
}

impl<T, A> Box<MaybeUninit<T>, A>
where
    A: Deallocate,
{
    pub unsafe fn assume_init(this: Self) -> Box<T, A> {
        let this = ManuallyDrop::new(this);
        let alloc = unsafe { ptr::read(&this.alloc) };

        Box {
            alloc,
            ptr: this.ptr.cast(),
            _p: PhantomData,
        }
    }

    pub fn write(mut this: Self, value: T) -> Box<T, A> {
        unsafe {
            this.as_mut_ptr().write(MaybeUninit::new(value));
            Box::assume_init(this)
        }
    }
}

pub unsafe trait SplitDeallocate: Allocate {
    type Deallocator: Deallocate;

    fn deallocator(&self) -> Self::Deallocator;
}
