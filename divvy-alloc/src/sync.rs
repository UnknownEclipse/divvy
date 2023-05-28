// use std::{alloc::Layout, marker::PhantomData, mem::ManuallyDrop, ptr::NonNull};

// use divvy_core::{Deallocate, NonZeroLayout};

// struct RcRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: RcState,
// {
//     inner: NonNull<Inner<T, D, S>>,
// }

// impl<T, D, S> RcRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: RcState,
// {
//     #[inline]
//     pub fn get(&self) -> &T {
//         &self.inner().value
//     }

//     #[inline]
//     pub fn as_ptr(&self) -> *const T {
//         todo!()
//     }
// }

// impl<T, D, S> RcRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: WeakState,
// {
//     pub fn downgrade(&self) -> WeakRaw<T, D, S> {
//         todo!()
//     }
// }

// impl<T, D, S> RcRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: RcState,
// {
//     #[inline]
//     fn inner(&self) -> &Inner<T, D, S> {
//         unsafe { self.inner.as_ref() }
//     }

//     #[inline]
//     unsafe fn drop_value(&self) {
//         unsafe { self.as_ptr().cast_mut().drop_in_place() };
//     }

//     #[cold]
//     unsafe fn dealloc(&self) {
//         unsafe {
//             self.drop_value();
//             dealloc(self.inner);
//         }
//     }
// }

// impl<T, D, S> Drop for RcRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: RcState,
// {
//     #[inline]
//     fn drop(&mut self) {
//         match self.inner().header.state.drop() {
//             DropResult::StrongRemain => {}
//             DropResult::LastStrong => unsafe { self.drop_value() },
//             DropResult::LastRef => unsafe { self.dealloc() },
//         }
//     }
// }

// struct WeakRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: WeakState,
// {
//     header: NonNull<Header<D, S>>,
//     _p: PhantomData<Inner<T, D, S>>,
// }

// impl<T, D, S> WeakRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: WeakState,
// {
//     fn header(&self) -> &Header<D, S> {
//         todo!()
//     }
// }

// impl<T, D, S> Drop for WeakRaw<T, D, S>
// where
//     T: ?Sized,
//     D: Deallocate,
//     S: WeakState,
// {
//     #[inline]
//     fn drop(&mut self) {
//         match self.inner().header.state.drop_ref() {
//             DropResult::StrongRemain => {}
//             DropResult::LastStrong => unsafe { self.drop_value() },
//             DropResult::LastRef => unsafe { self.dealloc() },
//         }
//     }
// }

// struct Header<D, S> {
//     state: S,
//     dealloc: ManuallyDrop<D>,
// }

// #[repr(C)]
// struct Inner<T, D, S>
// where
//     T: ?Sized,
// {
//     header: Header<D, S>,
//     value: ManuallyDrop<T>,
// }

// impl<T, D, S> Inner<T, D, S>
// where
//     T: ?Sized,
// {
//     #[inline]
//     pub fn layout(&self) -> Layout {
//         Layout::for_value(self)
//     }
// }

// unsafe fn dealloc<T, D, S>(mut inner: NonNull<Inner<T, D, S>>)
// where
//     T: ?Sized,
//     D: Deallocate,
// {
//     let r = unsafe { inner.as_mut() };
//     let layout = Layout::for_value(r);
//     let dealloc = ManuallyDrop::take(&mut r.header.dealloc);

//     if let Some(layout) = NonZeroLayout::new(layout) {
//         unsafe { dealloc.deallocate(inner.cast(), layout) };
//     }
// }

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// enum DropResult {
//     StrongRemain,
//     LastStrong,
//     LastRef,
// }

// unsafe trait RcState {
//     fn clone(&self);
//     fn drop(&self) -> DropResult;
// }

// unsafe trait WeakState: RcState {
//     fn downgrade(&self);
//     fn upgrade(&self) -> bool;
//     fn clone_weak(&self);
//     fn drop_weak(&self) -> DropResult;
// }

// struct ArcState {}
