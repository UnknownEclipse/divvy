use core::{
    alloc::Layout,
    fmt::{Debug, Display},
    hash::Hash,
    mem::{ManuallyDrop, MaybeUninit},
    ops::{Deref, DerefMut},
    pin::Pin,
    ptr::{self, NonNull},
};

use divvy_core::{AllocError, Allocator, Deallocator, NonZeroLayout};

pub struct Box<T, A>
where
    T: ?Sized,
    A: Deallocator,
{
    ptr: NonNull<T>,
    allocator: A,
}

impl<T, A> Box<T, A>
where
    A: Allocator,
{
    #[inline]
    pub fn try_new_uninit_in(allocator: A) -> Result<Box<MaybeUninit<T>, A>, AllocError> {
        let layout = Layout::new::<T>();

        let ptr = match NonZeroLayout::new(layout) {
            Some(layout) => allocator.allocate(layout)?.cast(),
            None => NonNull::dangling(),
        };

        Ok(Box { allocator, ptr })
    }

    #[inline]
    pub fn try_new_in(value: T, allocator: A) -> Result<Box<T, A>, AllocError> {
        let b = Box::try_new_uninit_in(allocator)?;
        Ok(Box::write(b, value))
    }

    #[inline]
    pub fn try_pin_in(value: T, allocator: A) -> Result<Pin<Box<T, A>>, AllocError> {
        let b = Box::try_new_in(value, allocator)?;
        unsafe { Ok(Pin::new_unchecked(b)) }
    }

    #[inline]
    pub fn new_uninit_in(allocator: A) -> Box<MaybeUninit<T>, A> {
        Box::try_new_uninit_in(allocator).expect("allocation failed")
    }

    #[inline]
    pub fn new_in(value: T, allocator: A) -> Box<T, A> {
        Box::try_new_in(value, allocator).expect("allocation failed")
    }

    #[inline]
    pub fn pin_in(value: T, allocator: A) -> Pin<Box<T, A>> {
        Box::try_pin_in(value, allocator).expect("allocation failed")
    }
}

impl<T, A> Box<T, A>
where
    A: Deallocator,
{
    pub fn write(mut b: Box<MaybeUninit<T>, A>, value: T) -> Box<T, A> {
        b.deref_mut().write(value);
        let (ptr, allocator) = Box::into_raw_with_allocator(b);
        unsafe { Box::from_raw_in(ptr.cast(), allocator) }
    }

    pub fn into_inner(b: Box<T, A>) -> T {
        let (ptr, allocator) = Box::into_raw_with_allocator(b);

        let layout = Layout::new::<T>();
        let value = unsafe { ptr.read() };
        if let Some(layout) = NonZeroLayout::new(layout) {
            unsafe { allocator.deallocate(NonNull::new_unchecked(ptr).cast(), layout) };
        }
        value
    }
}

impl<T, A> Box<MaybeUninit<T>, A>
where
    A: Deallocator,
{
    /// # Safety
    ///
    /// The value must have been previously initialized.
    pub unsafe fn assume_init(b: Box<MaybeUninit<T>, A>) -> Box<T, A> {
        let (ptr, allocator) = Box::into_raw_with_allocator(b);
        unsafe { Box::from_raw_in(ptr.cast(), allocator) }
    }
}

impl<T, A> Box<T, A>
where
    T: ?Sized,
    A: Deallocator,
{
    /// # Safety
    ///
    /// The pointer and allcoator must have been created by a previous call to
    /// `Box::into_raw_with_allocator`.
    #[inline]
    pub unsafe fn from_raw_in(ptr: *mut T, allocator: A) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(ptr) },
            allocator,
        }
    }

    #[inline]
    pub fn into_raw_with_allocator(this: Box<T, A>) -> (*mut T, A) {
        let this = ManuallyDrop::new(this);
        let ptr = this.ptr.as_ptr();
        let allocator = unsafe { ptr::read(&this.allocator) };
        (ptr, allocator)
    }

    pub fn allocator(b: &Box<T, A>) -> &A {
        &b.allocator
    }

    pub fn leak<'a>(b: Box<T, A>) -> &'a mut T
    where
        A: 'a,
    {
        let mut b = ManuallyDrop::new(b);
        unsafe { b.ptr.as_mut() }
    }
}

impl<T, A> Deref for Box<T, A>
where
    T: ?Sized,
    A: Deallocator,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A> DerefMut for Box<T, A>
where
    T: ?Sized,
    A: Deallocator,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T, A> Drop for Box<T, A>
where
    T: ?Sized,
    A: Deallocator,
{
    fn drop(&mut self) {
        let layout = Layout::for_value(self.deref());
        if let Some(layout) = NonZeroLayout::new(layout) {
            unsafe { self.allocator.deallocate(self.ptr.cast(), layout) };
        }
    }
}

impl<T, A> Debug for Box<T, A>
where
    T: ?Sized + Debug,
    A: Deallocator + Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: &T = self;
        f.debug_struct("Box")
            .field("value", &v)
            .field("allocator", &self.allocator)
            .finish()
    }
}

impl<T, A> Display for Box<T, A>
where
    T: ?Sized + Display,
    A: Deallocator,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let v: &T = self;
        Display::fmt(v, f)
    }
}

impl<T, A> Hash for Box<T, A>
where
    T: ?Sized + Hash,
    A: Deallocator + Debug,
{
    #[inline]
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.deref().hash(state)
    }
}

impl<T, A> Clone for Box<T, A>
where
    T: ?Sized + Clone,
    A: Allocator + Clone,
{
    fn clone(&self) -> Self {
        let value = self.deref().clone();
        let allocator = self.allocator.clone();
        Box::new_in(value, allocator)
    }
}
