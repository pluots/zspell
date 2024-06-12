#![allow(clippy::incompatible_msrv)]

use std::fs;
use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use zspell::bench::{affix_from_str, DictEntry, FlagType};
use zspell::{DictBuilder, Dictionary};

const TEXT: &str = "A Hare was mking fun of the Tortoise one day for being so slow.

Do you ever get anywhere? he asked with a mocking laugh.

Yes, replied the Tortoise, and I get there sooner than you think. I'll
run you a race and prove it.

The Hare was much amused at the iea of running a race with the Tortise,
but for the fun of the thing he agreed. So the Fox, who had consented to
act as judge, maarked the distance and started the runners off.

The Hare was soon far out of sight, and to make the Tortoise feel very
deeply how ridiculous it was for him to try a race with a Hare, he lay
down beside the course to take a nap until the Tortoise should catch up.

The Tortoise meanwhile kept going sloly but steadily, and, after a time,
passed the place where the Hare was sleeping. But the Hare slept on very
peacefully; and when at last he did wake up, the Tortoise was near the goal.
The Hare now ran his swiftest, but he could not overtaake the Tortoise
in time.";

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

fn fixture_create_en_dict() -> Dictionary {
    // Test that we correctly compile the short wordlist

    let aff_content = fs::read_to_string("../dictionaries/en_US.aff").unwrap();
    let dic_content = fs::read_to_string("../dictionaries/en_US.dic").unwrap();

    DictBuilder::new()
        .dict_str(black_box(&dic_content))
        .config_str(black_box(&aff_content))
        .build()
        .unwrap()
}

pub fn bench_parsers(c: &mut Criterion) {
    let aff_content = fs::read_to_string("../dictionaries/en_US.aff").unwrap();
    let dic_content = fs::read_to_string("../dictionaries/en_US.dic").unwrap();

    c.bench_function("Parse affix file", |b| {
        b.iter(|| black_box(affix_from_str(black_box(&aff_content)).unwrap()))
    });

    c.bench_function("Parse dict file", |b| {
        b.iter(|| {
            black_box(
                DictEntry::parse_all(black_box(&dic_content), black_box(FlagType::Utf8)).unwrap(),
            )
        })
    });
}

/// This test just creates a dictionary. The compiling is the slow step.
pub fn bench_dict_compile(c: &mut Criterion) {
    let aff_content = fs::read_to_string("../dictionaries/en_US.aff").unwrap();
    let dic_content = fs::read_to_string("../dictionaries/en_US.dic").unwrap();

    c.bench_function("Spellcheck: compile dictionary", |b| {
        b.iter(|| {
            black_box(
                DictBuilder::new()
                    .dict_str(black_box(&dic_content))
                    .config_str(black_box(&aff_content))
                    .build()
                    .unwrap(),
            )
        })
    });
}

pub fn bench_dict_simple(c: &mut Criterion) {
    let dict = fixture_create_en_dict();
    c.bench_function("Spellcheck: 1 correct word", |b| {
        b.iter(|| black_box(dict.check_word(black_box("turbidity's"))))
    });

    c.bench_function("Spellcheck: 1 incorrect word", |b| {
        b.iter(|| black_box(dict.check_word(black_box("turbiditated"))))
    });

    c.bench_function("Spellcheck: 15 correct words", |b| {
        b.iter(|| {
            for item in CONTAINS_LIST {
                black_box(dict.check(black_box(item)));
            }
        })
    });

    c.bench_function("Spellcheck: 15 incorrect words", |b| {
        b.iter(|| {
            for item in NOT_CONTAINS_LIST {
                black_box(dict.check(black_box(item)));
            }
        })
    });
}

pub fn bench_dict_paragraph(c: &mut Criterion) {
    let dict = fixture_create_en_dict();

    c.bench_function("Spellcheck: 188 word paragraph", |b| {
        b.iter(|| black_box(dict.check(black_box(TEXT))))
    });
}

criterion_group!(
    dict_integration,
    bench_parsers,
    bench_dict_compile,
    bench_dict_simple,
    bench_dict_paragraph,
    // bench_parallel,
);
criterion_main!(dict_integration);
