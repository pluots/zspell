use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use unicode_segmentation::UnicodeSegmentation;

const TESTSTR: &str = "the quick brown.   Fox Jum-ped -- where? 'over' (the) very-lazy dog";

// These aren't fair comparisons because they don't return indices
pub fn basic_splits(c: &mut Criterion) {
    c.bench_function("Split whitespace", |b| {
        b.iter(|| black_box(black_box(TESTSTR).split_whitespace().last().unwrap()))
    });
    c.bench_function("Split ascii whitespace", |b| {
        b.iter(|| black_box(black_box(TESTSTR).split_ascii_whitespace().last().unwrap()))
    });
}

pub fn segmentation(c: &mut Criterion) {
    c.bench_function("Simple segmentation", |b| {
        b.iter(|| {
            black_box(
                black_box(TESTSTR)
                    .split_word_bound_indices()
                    .last()
                    .unwrap(),
            )
        })
    });
    c.bench_function("Skip whitespace using all", |b| {
        b.iter(|| {
            black_box(
                black_box(TESTSTR)
                    .split_word_bound_indices()
                    .filter(|split| split.1.chars().all(|c| c.is_alphanumeric() || c == '-'))
                    .last()
                    .unwrap(),
            )
        })
    });
    c.bench_function("Skip whitespace using first", |b| {
        b.iter(|| {
            black_box(
                black_box(TESTSTR)
                    .split_word_bound_indices()
                    .filter(|split| {
                        let first = split.1.chars().next().unwrap();
                        first.is_alphanumeric() || first == '-'
                    })
                    .last()
                    .unwrap(),
            )
        })
    });
    c.bench_function("Skip whitespace using first nohyphen", |b| {
        b.iter(|| {
            black_box(
                black_box(TESTSTR)
                    .split_word_bound_indices()
                    .filter(|split| split.1.chars().next().unwrap().is_alphanumeric())
                    .last()
                    .unwrap(),
            )
        })
    });
}

// pub fn segmentation_peek(c: &mut Criterion) {
//     c.bench_function("Skip whitespace using first", |b| {
//         b.iter(|| {
//             black_box(
//                 black_box(TESTSTR)
//                     .split_word_bound_indices()
//                     .filter(|split| {
//                         let first = split.1.chars().next().unwrap();
//                         first.is_alphanumeric() || first == '-'
//                     })
//                     .last()
//                     .unwrap(),
//             )
//         })
//     });
// }

criterion_group!(word_splitter, basic_splits, segmentation);
criterion_main!(word_splitter);
