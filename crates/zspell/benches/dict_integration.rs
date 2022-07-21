use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use zspell::Dictionary;

fn fixture_create_en_dict() -> Dictionary {
    // Test that we correctly compile the short wordlist
    let mut dic = Dictionary::new();

    let aff_content = fs::read_to_string("../../dictionaries/en.aff").unwrap();
    let dic_content = fs::read_to_string("../../dictionaries/en.dic").unwrap();

    dic.config.load_from_str(aff_content.as_str()).unwrap();
    dic.load_dict_from_str(dic_content.as_str());
    dic.compile().unwrap();
    dic
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
pub fn dict_correct(dic: &Dictionary) {
    for item in CONTAINS_LIST {
        dic.check(item);
    }
}

#[inline]
pub fn dict_incorrect(dic: &Dictionary) {
    for item in NOT_CONTAINS_LIST {
        dic.check(item);
    }
}

/// This test just creates a dictionary. The compiling is the slow step.
pub fn bench_dict_compile(c: &mut Criterion) {
    c.bench_function("Spellcheck: compile dictionary", |b| {
        b.iter(|| fixture_create_en_dict())
    });
}

pub fn bench_dict_check_exists_single(c: &mut Criterion) {
    let dic = fixture_create_en_dict();
    c.bench_function("Spellcheck: 1 correct word", |b| {
        b.iter(|| dic.check(black_box("turbidity's")))
    });
}

pub fn bench_dict_check_not_exists_single(c: &mut Criterion) {
    let dic = fixture_create_en_dict();
    c.bench_function("Spellcheck: 1 incorrect word", |b| {
        b.iter(|| dic.check(black_box("turbiditated")))
    });
}

pub fn bench_dict_check_exists(c: &mut Criterion) {
    let dic = fixture_create_en_dict();
    c.bench_function("Spellcheck: 15 correct words", |b| {
        b.iter(|| dict_correct(black_box(&dic)))
    });
}

pub fn bench_dict_check_not_exists(c: &mut Criterion) {
    let dic = fixture_create_en_dict();
    c.bench_function("Spellcheck: 15 incorrect words", |b| {
        b.iter(|| dict_incorrect(black_box(&dic)))
    });
}

pub fn bench_dict_paragraph(c: &mut Criterion) {
    let dic = fixture_create_en_dict();
    let text = "A Hare was mking fun of the Tortoise one day for being so slow.

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

    let words = text.split_whitespace().collect::<Vec<&str>>();

    c.bench_function("Spellcheck: 188 word paragraph", |b| {
        b.iter(|| {
            words.iter().for_each(|s| {
                dic.check(s);
            })
        })
    });
}

criterion_group!(
    dict_integration,
    bench_dict_compile,
    bench_dict_check_exists_single,
    bench_dict_check_not_exists_single,
    bench_dict_check_exists,
    bench_dict_check_not_exists,
    bench_dict_paragraph,
);
criterion_main!(dict_integration);
