use std::{alloc::Layout, ptr::NonNull, sync::atomic::AtomicPtr};

use divvy_core::{Deallocate, NonZeroLayout};

pub struct SyncArena<A>
where
    A: Deallocate,
{
    shards: NonNull<[ArenaShard]>,
    backing: A,
}

impl<A> SyncArena<A>
where
    A: Deallocate,
{
    fn shards(&self) -> &[ArenaShard] {
        unsafe { self.shards.as_ref() }
    }

    fn shards_mut(&mut self) -> &mut [ArenaShard] {
        todo!()
    }
}

impl<A> Drop for SyncArena<A>
where
    A: Deallocate,
{
    fn drop(&mut self) {
        unsafe {
            let start: *mut ArenaShard = self.shards.as_ptr().cast();

            for i in 0..self.shards.len() {
                let ptr = start.add(i);

                (*ptr).dealloc(&self.backing);
                ptr.drop_in_place();
            }
        }

        let layout = Layout::array::<ArenaShard>(self.shards.len()).unwrap();
        let layout = NonZeroLayout::new(layout).unwrap();
        unsafe { self.backing.deallocate(self.shards.cast(), layout) };
    }
}

struct ArenaShard {
    current: AtomicPtr<()>,
}

impl ArenaShard {
    unsafe fn dealloc(&mut self, dealloc: &dyn Deallocate) {}
}
