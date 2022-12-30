//! Benchmarks for operations on datastructures that resemble operations we
//! might use in our spellchecker

#![allow(clippy::disallowed_types)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::hint::black_box;
use std::io::{self, BufRead};
use std::iter::FromIterator;

use criterion::{criterion_group, criterion_main, Criterion};
use hashbrown::{HashMap as HashBrownMap, HashSet as HashBrownSet};

// We will check all variables in these contains and contains false lists - we
// want a variety of names from throughout the set
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

static STR_REF: &str = "SOMETHING";

/// Load lines from a file
/// Strip the affix "/" directive
fn lines_loader() -> Vec<String> {
    let file = File::open("../../dictionaries/en_US.dic").unwrap();
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

type NestedVecMap<T1, T2> = Vec<(T1, Vec<T2>)>;

/// Take the results of `lines_loader` and create a map datatype
/// This replicates the data structure we store with some meta
fn map_loader() -> NestedVecMap<String, &'static str> {
    let lines = lines_loader();
    lines
        .iter()
        .map(|line| (line.clone(), vec![STR_REF]))
        .collect()
}

// Actual benchmark calling functions

pub fn bench_vec(c: &mut Criterion) {
    let vec: Vec<String> = lines_loader();

    c.bench_function("Vec contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(vec.iter().any(|x| x == black_box(item)));
            }
        })
    });

    c.bench_function("Vec contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(vec.iter().any(|x| x == black_box(item)));
            }
        })
    });
}

pub fn bench_btree(c: &mut Criterion) {
    let bt = BTreeSet::from_iter(lines_loader().into_iter());

    c.bench_function("BTree contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(bt.contains(black_box(item)));
            }
        })
    });

    c.bench_function("BTree contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(bt.contains(black_box(item)));
            }
        })
    });
}

pub fn bench_hashset(c: &mut Criterion) {
    let hs: HashSet<String> = HashSet::from_iter(lines_loader().into_iter());

    c.bench_function("HashSet contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hs.contains(black_box(item)));
            }
        })
    });

    c.bench_function("HashSet contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hs.contains(black_box(item)));
            }
        })
    });
}

pub fn bench_hashbrownset(c: &mut Criterion) {
    let hs: HashBrownSet<String> = HashBrownSet::from_iter(lines_loader().into_iter());

    c.bench_function("HashBrownSet contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hs.contains(black_box(item)));
            }
        })
    });

    c.bench_function("HashBrownSet contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hs.contains(black_box(item)));
            }
        })
    });
}

// Map type benchmarks

pub fn bench_vecmap(c: &mut Criterion) {
    let vm: NestedVecMap<_, _> = map_loader();

    c.bench_function("VecMap contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(vm.iter().any(|x| x.0 == black_box(item)));
            }
        })
    });

    c.bench_function("VecMap contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(vm.iter().any(|x| x.0 == black_box(item)));
            }
        })
    });

    c.bench_function("VecMap get true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(vm.iter().find(|x| x.0 == black_box(item)).map(|x| &x.1));
            }
        })
    });

    c.bench_function("VecMap get false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(vm.iter().find(|x| x.0 == black_box(item)).map(|x| &x.1));
            }
        })
    });
}

pub fn bench_btreemap(c: &mut Criterion) {
    let bt: BTreeMap<String, _> = BTreeMap::from_iter(map_loader().into_iter());

    c.bench_function("BTreeMap contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(bt.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("BTreeMap contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(bt.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("BTreeMap get true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(bt.get(black_box(item)));
            }
        })
    });

    c.bench_function("BTreeMap get false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(bt.get(black_box(item)));
            }
        })
    });
}

pub fn bench_hashmap(c: &mut Criterion) {
    let hm: HashMap<String, _> = HashMap::from_iter(map_loader().into_iter());

    c.bench_function("HashMap contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hm.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("HashMap contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hm.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("HashMap get true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hm.get(black_box(item)));
            }
        })
    });

    c.bench_function("HashMap get false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hm.get(black_box(item)));
            }
        })
    });
}

pub fn bench_hashbrownmap(c: &mut Criterion) {
    let hm: HashBrownMap<String, _> = HashBrownMap::from_iter(map_loader().into_iter());

    c.bench_function("HashBrownMap contains true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hm.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("HashBrownMap contains false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hm.contains_key(black_box(item)));
            }
        })
    });

    c.bench_function("HashBrownMap get true", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(hm.get(black_box(item)));
            }
        })
    });

    c.bench_function("HashBrownMap get false", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(hm.get(black_box(item)));
            }
        })
    });
}

criterion_group!(
    datastructure,
    bench_vec,
    bench_btree,
    bench_hashset,
    bench_hashbrownset,
    bench_vecmap,
    bench_btreemap,
    bench_hashmap,
    bench_hashbrownmap
);
criterion_main!(datastructure);
