//! Benchmark the difference between contains & `binary_search`es, intended

use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

pub fn benches(c: &mut Criterion) {
    const EMPTY: [&str; 0] = [];
    const SORT1: [&str; 1] = ["A"];
    const SORT3: [&str; 3] = ["A", "B", "C"];
    const SORT10: [&str; 10] = ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"];
    const UNSORT3: [&str; 3] = ["C", "A", "B"];
    const UNSORT10: [&str; 10] = ["B", "F", "C", "A", "I", "J", "E", "D", "H", "G"];

    c.bench_function("Empty: `contains`", |b| {
        b.iter(|| black_box(&EMPTY).contains(black_box(&"A")))
    });

    c.bench_function("Empty: `binary_search`", |b| {
        b.iter(|| black_box(&EMPTY).binary_search(black_box(&"A")).is_ok())
    });

    c.bench_function("1x: `contains`", |b| {
        b.iter(|| black_box(&SORT1).contains(black_box(&"A")))
    });

    c.bench_function("1x: `binary_search`", |b| {
        b.iter(|| black_box(&SORT1).binary_search(black_box(&"A")).is_ok())
    });

    c.bench_function("3 sorted: `contains`", |b| {
        b.iter(|| black_box(SORT3).contains(black_box(&"B")))
    });

    c.bench_function("3 sorted: `binary_search`", |b| {
        b.iter(|| black_box(SORT3).binary_search(black_box(&"B")).is_ok())
    });

    c.bench_function("10 sorted: `contains` early", |b| {
        b.iter(|| black_box(SORT10).contains(black_box(&"B")))
    });

    c.bench_function("10 sorted: `binary_search` early", |b| {
        b.iter(|| black_box(SORT10).binary_search(black_box(&"B")).is_ok())
    });

    c.bench_function("10 sorted: `contains` mid", |b| {
        b.iter(|| black_box(SORT10).contains(black_box(&"G")))
    });

    c.bench_function("10 sorted: `binary_search` mid", |b| {
        b.iter(|| black_box(SORT10).binary_search(black_box(&"G")).is_ok())
    });

    c.bench_function("10 sorted: `contains` late", |b| {
        b.iter(|| black_box(SORT10).contains(black_box(&"J")))
    });

    c.bench_function("10 sorted: `binary_search` late", |b| {
        b.iter(|| black_box(SORT10).binary_search(black_box(&"J")).is_ok())
    });

    c.bench_function("3 unsorted: `contains`", |b| {
        b.iter(|| black_box(SORT3).contains(black_box(&"B")))
    });

    c.bench_function("3 unsorted: `binary_search`", |b| {
        b.iter(|| {
            let mut arr = black_box(UNSORT3);
            arr.sort_unstable();
            black_box(arr).binary_search(black_box(&"B")).is_ok()
        })
    });

    c.bench_function("10 unsorted: `contains`", |b| {
        b.iter(|| black_box(SORT10).contains(black_box(&"G")))
    });

    c.bench_function("10 unsorted: `binary_search`", |b| {
        b.iter(|| {
            let mut arr = black_box(UNSORT10);
            arr.sort_unstable();
            black_box(arr).binary_search(black_box(&"G")).is_ok()
        })
    });
}

criterion_group!(slice_contains, benches,);
criterion_main!(slice_contains);
