//! Benchmarks for our flag map, which will be a small small mapping of integers
//! to integers
//!
//! Findings:
//!
//! I ran this benchmark to figure a way of keeping our keys in the table. In
//! the end, the ordering seemed to be (fastest to slowest):
//!
//! - HashBrownMap
//! - BTreeMap
//! - HashMap
//! - Vec
//!
//! For now, I am going to go with a BTreeMap because I think there may be some
//! eventual benefits of keeping sortability. We should re benchmark this in
//! situ (should only affect compile times)

#![allow(clippy::disallowed_types)]

use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use hashbrown::HashMap as HashBrownMap;
use rand::random;

const CAP_SHORT: usize = 6;
const CAP_MED: usize = 24;
const CAP_LONG: usize = 80;
const CAP_XLONG: usize = 400;

type ValType = [i32; 8];

fn make_vec(len: usize) -> (Vec<(u32, ValType)>, Vec<u32>) {
    let mut ret = Vec::with_capacity(len);
    for _ in 0..len {
        ret.push((random(), random()))
    }
    ret.sort_unstable();
    let keys = vec![ret[0].0, ret[len / 3].0, ret[len * 2 / 3].0, ret[len - 1].0];

    (ret, keys)
}

pub fn bench_vec_map(c: &mut Criterion) {
    let (v_short, keys_short) = make_vec(CAP_SHORT);
    let (v_med, keys_med) = make_vec(CAP_MED);
    let (v_long, keys_long) = make_vec(CAP_LONG);
    let (v_xlong, keys_xlong) = make_vec(CAP_XLONG);

    c.bench_function("Vec short get", |b| {
        b.iter(|| {
            for key in &keys_short {
                black_box(
                    black_box(&v_short)
                        .iter()
                        .find(|(k, _)| *k == black_box(*key))
                        .map(|(_, v)| v),
                );
            }
        })
    });
    c.bench_function("Vec med get", |b| {
        b.iter(|| {
            for key in &keys_med {
                black_box(
                    black_box(&v_med)
                        .iter()
                        .find(|(k, _)| *k == black_box(*key))
                        .map(|(_, v)| v),
                );
            }
        })
    });
    c.bench_function("Vec long get", |b| {
        b.iter(|| {
            for key in &keys_long {
                black_box(
                    black_box(&v_long)
                        .iter()
                        .find(|(k, _)| *k == black_box(*key))
                        .map(|(_, v)| v),
                );
            }
        })
    });
    c.bench_function("Vec xlong get", |b| {
        b.iter(|| {
            for key in &keys_xlong {
                black_box(
                    black_box(&v_xlong)
                        .iter()
                        .find(|(k, _)| *k == black_box(*key))
                        .map(|(_, v)| v),
                );
            }
        })
    });
}

pub fn bench_vec_map_binsearch(c: &mut Criterion) {
    let (v_short, keys_short) = make_vec(CAP_SHORT);
    let (v_med, keys_med) = make_vec(CAP_MED);
    let (v_long, keys_long) = make_vec(CAP_LONG);
    let (v_xlong, keys_xlong) = make_vec(CAP_XLONG);

    c.bench_function("Vec short binsearch", |b| {
        b.iter(|| {
            for key in &keys_short {
                black_box(
                    black_box(&v_short)
                        .binary_search_by_key(black_box(key), |&(k, _)| k)
                        .map(|idx| v_short[idx])
                        .ok(),
                );
            }
        })
    });
    c.bench_function("Vec med binsearch", |b| {
        b.iter(|| {
            for key in &keys_med {
                black_box(
                    black_box(&v_med)
                        .binary_search_by_key(black_box(key), |&(k, _)| k)
                        .map(|idx| v_med[idx])
                        .ok(),
                );
            }
        })
    });
    c.bench_function("Vec long binsearch", |b| {
        b.iter(|| {
            for key in &keys_long {
                black_box(
                    black_box(&v_long)
                        .binary_search_by_key(black_box(key), |&(k, _)| k)
                        .map(|idx| v_long[idx])
                        .ok(),
                );
            }
        })
    });
    c.bench_function("Vec xlong binsearch", |b| {
        b.iter(|| {
            for key in &keys_xlong {
                black_box(
                    black_box(&v_xlong)
                        .binary_search_by_key(black_box(key), |&(k, _)| k)
                        .map(|idx| v_xlong[idx])
                        .ok(),
                );
            }
        })
    });
}

pub fn bench_hash_map(c: &mut Criterion) {
    let (v_short, keys_short) = make_vec(CAP_SHORT);
    let (v_med, keys_med) = make_vec(CAP_MED);
    let (v_long, keys_long) = make_vec(CAP_LONG);
    let (v_xlong, keys_xlong) = make_vec(CAP_XLONG);

    let map_short: HashMap<u32, ValType> = HashMap::from_iter(v_short);
    let map_med: HashMap<u32, ValType> = HashMap::from_iter(v_med);
    let map_long: HashMap<u32, ValType> = HashMap::from_iter(v_long);
    let map_xlong: HashMap<u32, ValType> = HashMap::from_iter(v_xlong);

    c.bench_function("HashMap short get", |b| {
        b.iter(|| {
            for key in &keys_short {
                black_box(black_box(&map_short).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashMap med get", |b| {
        b.iter(|| {
            for key in &keys_med {
                black_box(black_box(&map_med).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashMap long get", |b| {
        b.iter(|| {
            for key in &keys_long {
                black_box(black_box(&map_long).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashMap xlong get", |b| {
        b.iter(|| {
            for key in &keys_xlong {
                black_box(black_box(&map_xlong).get(black_box(key)));
            }
        })
    });
}

pub fn bench_hashbrown_map(c: &mut Criterion) {
    let (v_short, keys_short) = make_vec(CAP_SHORT);
    let (v_med, keys_med) = make_vec(CAP_MED);
    let (v_long, keys_long) = make_vec(CAP_LONG);
    let (v_xlong, keys_xlong) = make_vec(CAP_XLONG);

    let map_short: HashBrownMap<u32, ValType> = HashBrownMap::from_iter(v_short);
    let map_med: HashBrownMap<u32, ValType> = HashBrownMap::from_iter(v_med);
    let map_long: HashBrownMap<u32, ValType> = HashBrownMap::from_iter(v_long);
    let map_xlong: HashMap<u32, ValType> = HashMap::from_iter(v_xlong);

    c.bench_function("HashBrownMap short get", |b| {
        b.iter(|| {
            for key in &keys_short {
                black_box(black_box(&map_short).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashBrownMap med get", |b| {
        b.iter(|| {
            for key in &keys_med {
                black_box(black_box(&map_med).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashBrownMap long get", |b| {
        b.iter(|| {
            for key in &keys_long {
                black_box(black_box(&map_long).get(black_box(key)));
            }
        })
    });
    c.bench_function("HashBrownMap xlong get", |b| {
        b.iter(|| {
            for key in &keys_xlong {
                black_box(black_box(&map_xlong).get(black_box(key)));
            }
        })
    });
}

pub fn bench_btree_map(c: &mut Criterion) {
    let (v_short, keys_short) = make_vec(CAP_SHORT);
    let (v_med, keys_med) = make_vec(CAP_MED);
    let (v_long, keys_long) = make_vec(CAP_LONG);
    let (v_xlong, keys_xlong) = make_vec(CAP_XLONG);

    let map_short: BTreeMap<u32, ValType> = BTreeMap::from_iter(v_short);
    let map_med: BTreeMap<u32, ValType> = BTreeMap::from_iter(v_med);
    let map_long: BTreeMap<u32, ValType> = BTreeMap::from_iter(v_long);
    let map_xlong: HashMap<u32, ValType> = HashMap::from_iter(v_xlong);

    c.bench_function("BTreeMap short get", |b| {
        b.iter(|| {
            for key in &keys_short {
                black_box(black_box(&map_short).get(black_box(key)));
            }
        })
    });
    c.bench_function("BTreeMap med get", |b| {
        b.iter(|| {
            for key in &keys_med {
                black_box(black_box(&map_med).get(black_box(key)));
            }
        })
    });
    c.bench_function("BTreeMap long get", |b| {
        b.iter(|| {
            for key in &keys_long {
                black_box(black_box(&map_long).get(black_box(key)));
            }
        })
    });
    c.bench_function("BTreeMap xlong get", |b| {
        b.iter(|| {
            for key in &keys_xlong {
                black_box(black_box(&map_xlong).get(black_box(key)));
            }
        })
    });
}

criterion_group!(
    small_map,
    bench_vec_map,
    bench_vec_map_binsearch,
    bench_hash_map,
    bench_hashbrown_map,
    bench_btree_map,
);
criterion_main!(small_map);
