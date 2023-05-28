use std::alloc::{alloc, Layout};

use bumpalo::Bump;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use divvy::{Arena, Leak, Os};
use divvy_core::{Allocate, NonZeroLayout};

fn bench_arena(c: &mut Criterion) {
    const COUNT: usize = 1 << 20;

    c.bench_with_input(BenchmarkId::new("bumpalo", COUNT), &COUNT, |b, &count| {
        b.iter_batched_ref(
            Bump::new,
            |b| {
                let result = b.alloc_layout(Layout::new::<usize>());
                black_box(result);
            },
            criterion::BatchSize::LargeInput,
        );
    });

    c.bench_with_input(BenchmarkId::new("arena", COUNT), &COUNT, |b, &count| {
        b.iter_batched_ref(
            || Arena::new(Leak::new(Os)),
            |b| {
                let layout = NonZeroLayout::new(Layout::new::<usize>()).unwrap();
                let result = b.allocate(layout);
                black_box(result).expect("alloc error");
            },
            criterion::BatchSize::LargeInput,
        );
    });

    c.bench_function("GlobalAlloc", |b| {
        b.iter_batched_ref(
            || {},
            |_| {
                let layout = Layout::new::<usize>();
                let result = unsafe { alloc(layout) };
                black_box(result);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_arena);
criterion_main!(benches);
