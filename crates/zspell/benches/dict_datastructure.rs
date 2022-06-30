use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::FromIterator;

/// Load lines from a file
/// Strip the affix "/" directive
fn lines_loader() -> Vec<String> {
    let file = File::open("../../dictionaries/en.dic").unwrap();
    let lines = io::BufReader::new(file).lines();

    let mut v: Vec<String> = Vec::new();

    for line in lines {
        v.push(line.unwrap().split('/').next().unwrap().to_string());
    }

    v
}

pub fn hash_setup() -> HashSet<String> {
    let hs = HashSet::from_iter(lines_loader().into_iter());
    hs
}

pub fn btree_setup() -> BTreeSet<String> {
    let bt = BTreeSet::from_iter(lines_loader().into_iter());
    bt
}

const CONTAINS_LIST: [&str; 15] = [
    "Accenture",
    "Curie",
    "Gujranwala",
    "Hesperus",
    "Juneau",
    "Lakeland",
    "Mephistopheles",
    "O'Connell",
    "Sweden",
    "Sarajevo",
    "sweptback",
    "tigerish",
    "Vespucci",
    "zymurgy",
    "0",
];

const NOT_CONTAINS_LIST: [&str; 15] = [
    "aaaaaa",
    "Curied",
    "gujranwalda",
    "Hesperuds",
    "Junaeau",
    "Lakaeland",
    "Mepsifstopheles",
    "OFonnell",
    "Swayden",
    "Sarajayovo",
    "sweptabback",
    "tigerstripeish",
    "Vespucki",
    "zzzzzzz",
    "000000",
];

#[inline]
pub fn btree_contains(obj: &BTreeSet<String>) {
    for item in CONTAINS_LIST {
        obj.contains(item);
    }
}

#[inline]
pub fn hash_contains(obj: &HashSet<String>) {
    for item in CONTAINS_LIST {
        obj.contains(item);
    }
}

#[inline]
pub fn btree_not_contains(obj: &BTreeSet<String>) {
    for item in NOT_CONTAINS_LIST {
        obj.contains(item);
    }
}

#[inline]
pub fn hash_not_contains(obj: &HashSet<String>) {
    for item in NOT_CONTAINS_LIST {
        obj.contains(item);
    }
}

#[inline]
pub fn btree_iterator(obj: &BTreeSet<String>) {
    let _v: Vec<&String> = obj.iter().collect();
}

#[inline]
pub fn hash_iterator(obj: &HashSet<String>) {
    let _v: Vec<&String> = obj.iter().collect();
}

// Actual benchmark calling functions

pub fn bench_btree_contains(c: &mut Criterion) {
    let bt = btree_setup();
    c.bench_function("BTree Contains", |b| b.iter(|| btree_contains(&bt)));
}

pub fn bench_btree_not_contains(c: &mut Criterion) {
    let bt = btree_setup();
    c.bench_function("BTree Not Contains", |b| b.iter(|| btree_not_contains(&bt)));
}

pub fn bench_btree_iter(c: &mut Criterion) {
    let bt = btree_setup();
    c.bench_function("BTree Iter", |b| b.iter(|| btree_iterator(&bt)));
}
pub fn hash_btree_contains(c: &mut Criterion) {
    let hs = hash_setup();
    c.bench_function("Hash Contains", |b| b.iter(|| hash_contains(&hs)));
}

pub fn hash_btree_not_contains(c: &mut Criterion) {
    let hs = hash_setup();
    c.bench_function("Hash Not Contains", |b| b.iter(|| hash_not_contains(&hs)));
}

pub fn hash_btree_iter(c: &mut Criterion) {
    let hs = hash_setup();
    c.bench_function("Hash Iter", |b| b.iter(|| hash_iterator(&hs)));
}

criterion_group!(
    dict_datastructure,
    bench_btree_contains,
    bench_btree_not_contains,
    bench_btree_iter,
    hash_btree_contains,
    hash_btree_not_contains,
    hash_btree_iter
);
criterion_main!(dict_datastructure);
