use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hashbrown;
use std::collections::{BTreeSet, HashSet};
use std::fs::File;
use std::io::{self, BufRead};
use std::iter::FromIterator;

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

/// Load lines from a file
/// Strip the affix "/" directive
fn lines_loader() -> Vec<String> {
    let file = File::open("../../dictionaries/en.dic").unwrap();
    let lines = io::BufReader::new(file).lines();

    let mut v: Vec<String> = Vec::new();

    for line in lines {
        v.push(line.unwrap().split('/').next().unwrap().to_owned());
    }

    // Validate items
    for item in CONTAINS_LIST {
        assert!(v.contains(&item.to_string()))
    }
    for item in NOT_CONTAINS_LIST {
        assert!(!v.contains(&item.to_string()))
    }

    v
}

// Actual benchmark calling functions

pub fn bench_vec(c: &mut Criterion) {
    let vec = lines_loader();

    c.bench_function("Vec Contains", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                vec.contains(&item.to_string());
            }
        })
    });

    c.bench_function("Vec Not Contains", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                vec.contains(&item.to_string());
            }
        })
    });

    c.bench_function("Vec Collect", |b| {
        b.iter(|| vec.iter().collect::<Vec<&String>>())
    });
}

pub fn bench_btree(c: &mut Criterion) {
    let bt = BTreeSet::from_iter(lines_loader().into_iter());

    c.bench_function("BTree Contains", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                bt.contains(item);
            }
        })
    });

    c.bench_function("BTree Not Contains", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                bt.contains(item);
            }
        })
    });

    c.bench_function("BTree Collect", |b| {
        b.iter(|| bt.iter().collect::<Vec<&String>>())
    });
}

pub fn bench_hashset(c: &mut Criterion) {
    let hs: HashSet<String> = HashSet::from_iter(lines_loader().into_iter());

    c.bench_function("Hash Contains", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                hs.contains(item);
            }
        })
    });

    c.bench_function("Hash Not Contains", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                hs.contains(item);
            }
        })
    });

    c.bench_function("Hash Collect", |b| {
        b.iter(|| hs.iter().collect::<Vec<&String>>())
    });
}

pub fn bench_hashbrownset(c: &mut Criterion) {
    let hs: hashbrown::HashSet<String> = hashbrown::HashSet::from_iter(lines_loader().into_iter());

    c.bench_function("Hashbrown Contains", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                hs.contains(item);
            }
        })
    });

    c.bench_function("Hashbrown Not Contains", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                hs.contains(item);
            }
        })
    });

    c.bench_function("Hashbrown Collect", |b| {
        b.iter(|| hs.iter().collect::<Vec<&String>>())
    });
}

criterion_group!(
    datastructure,
    bench_vec,
    bench_btree,
    bench_hashset,
    bench_hashbrownset
);
criterion_main!(datastructure);
