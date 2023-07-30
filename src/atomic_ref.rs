use core::{cell::UnsafeCell, marker::PhantomData, sync::atomic::AtomicU32};

use divvy_core::{Allocate, Deallocate, Grow, Shrink};

trait AllocFull: Allocate + Deallocate + Grow + Shrink {}

pub struct AtomicRef<'a> {
    stamp: AtomicU32,
    ptr: UnsafeCell<*const dyn AllocFull>,
    _p: PhantomData<&'a dyn AllocFull>,
}
