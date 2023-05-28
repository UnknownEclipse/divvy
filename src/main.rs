use std::{alloc::Layout, hint::black_box, println, time::Instant};

use bumpalo::Bump;
use divvy::{Arena, Global, Leak};
use divvy_core::{Allocate, NonZeroLayout};
use divvy_cpp::NewDelete;

fn main() {
    let arena = Arena::new(Leak::new(Global));
    let bump = Bump::new();

    const SIZE: usize = 1 << 28;
    let size = black_box(SIZE);

    let layout = Layout::new::<usize>();
    let nonzero_layout = NonZeroLayout::new(layout).unwrap();

    let start = Instant::now();
    for _ in 0..size {
        black_box(arena.allocate(nonzero_layout)).unwrap();
    }
    let end = Instant::now();
    println!("arena took {:?}", (end - start));

    let start = Instant::now();
    for _ in 0..size {
        black_box(bump.alloc_layout(layout));
    }
    let end = Instant::now();
    println!("bump took {:?}", (end - start));

    // let start = Instant::now();
    // for _ in 0..SIZE {
    //     let ptr = unsafe { std::alloc::alloc(layout) };
    //     black_box(ptr);
    // }
    // let end = Instant::now();
    // println!("std took {:?}", (end - start) / SIZE as u32);
}
