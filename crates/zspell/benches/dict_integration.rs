#![allow(unused)]
use std::fs;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
// use stringmetrics::{levenshtein, levenshtein_limit};
// use zspell::Dictionary;

// const TEXT: &str = "A Hare was mking fun of the Tortoise one day for being so slow.

// Do you ever get anywhere? he asked with a mocking laugh.

// Yes, replied the Tortoise, and I get there sooner than you think. I'll
// run you a race and prove it.

// The Hare was much amused at the iea of running a race with the Tortise,
// but for the fun of the thing he agreed. So the Fox, who had consented to
// act as judge, maarked the distance and started the runners off.

// The Hare was soon far out of sight, and to make the Tortoise feel very
// deeply how ridiculous it was for him to try a race with a Hare, he lay
// down beside the course to take a nap until the Tortoise should catch up.

// The Tortoise meanwhile kept going sloly but steadily, and, after a time,
// passed the place where the Hare was sleeping. But the Hare slept on very
// peacefully; and when at last he did wake up, the Tortoise was near the goal.
// The Hare now ran his swiftest, but he could not overtaake the Tortoise
// in time.";

// const CONTAINS_LIST: [&str; 15] = [
//     "Accenture",
//     "Curie",
//     "Gujranwala",
//     "Hesperus",
//     "Juneau",
//     "Lakeland",
//     "Mephistopheles",
//     "O'Connell",
//     "Sweden",
//     "Sarajevo",
//     "sweptback",
//     "tigerish",
//     "Vespucci",
//     "zymurgy",
//     "0",
// ];

// const NOT_CONTAINS_LIST: [&str; 15] = [
//     "aaaaaa",
//     "Curied",
//     "gujranwalda",
//     "Hesperuds",
//     "Junaeau",
//     "Lakaeland",
//     "Mepsifstopheles",
//     "OFonnell",
//     "Swayden",
//     "Sarajayovo",
//     "sweptabback",
//     "tigerstripeish",
//     "Vespucki",
//     "zzzzzzz",
//     "000000",
// ];

// fn fixture_create_en_dict() -> Dictionary {
//     // Test that we correctly compile the short wordlist
//     let mut dic = Dictionary::new();

//     let aff_content = fs::read_to_string("../../dictionaries/en_US.aff").unwrap();
//     let dic_content = fs::read_to_string("../../dictionaries/en_US.dic").unwrap();

//     dic.config.load_from_str(aff_content.as_str()).unwrap();
//     dic.load_dict_from_str(dic_content.as_str()).unwrap();
//     dic.compile().unwrap();
//     dic
// }

// /// This test just creates a dictionary. The compiling is the slow step.
// pub fn bench_dict_compile(c: &mut Criterion) {
//     c.bench_function("Spellcheck: compile dictionary", |b| {
//         b.iter(fixture_create_en_dict)
//     });
// }

// pub fn bench_dict_simple(c: &mut Criterion) {
//     let dic = fixture_create_en_dict();
//     c.bench_function("Spellcheck: 1 correct word", |b| {
//         b.iter(|| dic.check(black_box("turbidity's")))
//     });

//     c.bench_function("Spellcheck: 1 incorrect word", |b| {
//         b.iter(|| dic.check(black_box("turbiditated")))
//     });

//     c.bench_function("Spellcheck: 15 correct words", |b| {
//         b.iter(|| {
//             for item in CONTAINS_LIST {
//                 dic.check(item).unwrap();
//             }
//         })
//     });

//     c.bench_function("Spellcheck: 15 incorrect words", |b| {
//         b.iter(|| {
//             for item in NOT_CONTAINS_LIST {
//                 dic.check(item).unwrap();
//             }
//         })
//     });
// }

// pub fn bench_dict_paragraph(c: &mut Criterion) {
//     let dic = fixture_create_en_dict();

//     let words = TEXT.split_whitespace().collect::<Vec<&str>>();

//     c.bench_function("Spellcheck: 188 word paragraph", |b| {
//         b.iter(|| {
//             words.iter().for_each(|s| {
//                 dic.check(s).unwrap();
//             })
//         })
//     });
// }

// pub fn bench_lev(c: &mut Criterion) {
//     let dic = fixture_create_en_dict();
//     let word_items: Vec<&str> = dic
//         .iter_wordlist_items()
//         .unwrap()
//         .map(|s| s.as_str())
//         .collect();

//     c.bench_function("Lev nonparallel", |b| {
//         b.iter(|| {
//             word_items
//                 .iter()
//                 .map(|s| levenshtein(s, "turbiditated"))
//                 .min()
//         })
//     });

//     c.bench_function("Lev limit nonparallel", |b| {
//         b.iter(|| {
//             word_items
//                 .iter()
//                 .map(|s| levenshtein_limit(s, "turbiditated", 4))
//                 .min()
//         })
//     });
// }

pub fn tmp(c: &mut Criterion) {}

criterion_group!(
    dict_integration,
    tmp /* bench_dict_compile,
         * bench_dict_simple,
         * bench_dict_paragraph,
         * bench_parallel,
         * bench_lev */
);
criterion_main!(dict_integration);
