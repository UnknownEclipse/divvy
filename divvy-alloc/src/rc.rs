use std::{
    marker::PhantomData,
    ops::Deref,
    ptr::{addr_of, NonNull},
    sync::atomic::{AtomicUsize, Ordering},
};

use divvy_core::Deallocate;

pub type Arc<T, D> = RcRaw<T, AtomicState, D>;

#[derive(Debug)]
pub struct AtomicState {
    strong: AtomicUsize,
}

unsafe impl State for AtomicState {
    #[inline]
    fn clone(&self) {
        self.strong.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    fn drop(&self) -> Option<DropKind> {
        let n = self.strong.fetch_sub(1, Ordering::Relaxed);
        if n == 0 {
            Some(DropKind::All)
        } else {
            None
        }
    }
}

pub struct RcRaw<T, S, D>
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
    inner: NonNull<RcInner<T, S, D>>,
    _p: PhantomData<RcInner<T, S, D>>,
}

impl<T, S, D> Deref for RcRaw<T, S, D>
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*RcInner::as_ptr(self.inner) }
    }
}

impl<T, S, D> Drop for RcRaw<T, S, D>
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
    #[inline]
    fn drop(&mut self) {
        unsafe { RcInner::drop(self.inner) };
    }
}

impl<T, S, D> Clone for RcRaw<T, S, D>
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
    #[inline]
    fn clone(&self) -> Self {
        unsafe { RcInner::clone(self.inner) };

        Self {
            inner: self.inner,
            _p: PhantomData,
        }
    }
}

#[derive(Debug)]
struct RcInner<T, S, D>
where
    T: ?Sized,
{
    header: Header<S, D>,
    value: T,
}

impl<T, S, D> RcInner<T, S, D>
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
    unsafe fn as_ptr(ptr: NonNull<Self>) -> *const T {
        unsafe { addr_of!((*ptr.as_ptr()).value) }
    }

    unsafe fn drop(ptr: NonNull<Self>) {
        todo!()
    }

    unsafe fn clone(ptr: NonNull<Self>) {
        todo!()
    }
}

impl<T, S, D> RcInner<T, S, D>
where
    T: ?Sized,
    S: WeakState,
    D: Deallocate,
{
    unsafe fn drop_weak(ptr: NonNull<Self>) {
        todo!()
    }

    unsafe fn clone_weak(ptr: NonNull<Self>) {
        todo!()
    }
}

#[derive(Debug)]
struct Header<S, D> {
    state: S,
    dealloc: D,
}

unsafe fn deallocate<T, S, D>(ptr: NonNull<RcInner<T, S, D>>)
where
    T: ?Sized,
    S: State,
    D: Deallocate,
{
}

pub unsafe trait State {
    fn clone(&self);
    fn drop(&self) -> Option<DropKind>;
}

pub unsafe trait WeakState: State {
    fn downgrade(&self);
    fn upgrade(&self) -> bool;
    fn drop_weak(&self) -> bool;
    fn clone_weak(&self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropKind {
    All,
    Value,
}
