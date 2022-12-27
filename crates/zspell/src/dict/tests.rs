//! Tests for a dict file

use pretty_assertions::assert_eq;

use super::parser::{parse_personal_dict, DictEntry};
use super::*;
use crate::morph::MorphInfo;

#[test]
fn test_dict_entry_ok() {
    let f1 = FlagType::Utf8;
    let f2 = FlagType::Ascii;

    let s1 = "abcd";
    let s2 = "abcd # comment";
    let s3 = "abcd/ABC";
    let s4 = "abcd/ABC # comment";
    let s5 = "abcd/ABC ip:m1 tp:m2";
    let s6 = "abcd/ABC ip:m1 tp:m2 # comment";
    let s7 = "abcd ip:m1 tp:m2";
    let s8 = "abcd ip:m1 tp:m2 # comment";

    let r1 = DictEntry::new("abcd".to_owned(), &[], Vec::new());
    let r2 = DictEntry::new(
        "abcd".to_owned(),
        &['A' as u32, 'B' as u32, 'C' as u32],
        Vec::new(),
    );
    let r3 = DictEntry::new(
        "abcd".to_owned(),
        &['A' as u32, 'B' as u32, 'C' as u32],
        vec![
            MorphInfo::InflecPfx("m1".to_owned()),
            MorphInfo::TermPfx("m2".to_owned()),
        ],
    );
    let r4 = DictEntry::new(
        "abcd".to_owned(),
        &[],
        vec![
            MorphInfo::InflecPfx("m1".to_owned()),
            MorphInfo::TermPfx("m2".to_owned()),
        ],
    );

    assert_eq!(DictEntry::parse_str(s1, f1, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_str(s2, f1, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_str(s3, f1, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_str(s4, f1, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_str(s5, f1, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_str(s6, f1, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_str(s7, f1, 0), Ok(r4.clone()));
    assert_eq!(DictEntry::parse_str(s8, f1, 0), Ok(r4.clone()));

    assert_eq!(DictEntry::parse_str(s1, f2, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_str(s2, f2, 0), Ok(r1));
    assert_eq!(DictEntry::parse_str(s3, f2, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_str(s4, f2, 0), Ok(r2));
    assert_eq!(DictEntry::parse_str(s5, f2, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_str(s6, f2, 0), Ok(r3));
    assert_eq!(DictEntry::parse_str(s7, f2, 0), Ok(r4.clone()));
    assert_eq!(DictEntry::parse_str(s8, f2, 0), Ok(r4));
}

#[test]
fn test_personal_entry_ok() {
    let s1 = "abcd # comment";
    let s2 = "abcd/ABC # comment";
    let s3 = "*abcd/ABC # comment";
    let s4 = "abcd/ABC ip:m1 tp:m2 # comment";

    let r1 = PersonalEntry::new("abcd", None, Vec::new(), false);
    let r2 = PersonalEntry::new("abcd", Some("ABC"), Vec::new(), false);
    let r3 = PersonalEntry::new("abcd", Some("ABC"), Vec::new(), true);
    let r4 = PersonalEntry::new(
        "abcd",
        Some("ABC"),
        vec![
            MorphInfo::InflecPfx("m1".to_owned()),
            MorphInfo::TermPfx("m2".to_owned()),
        ],
        false,
    );

    assert_eq!(PersonalEntry::parse_str(s1, 0), Ok(r1));
    assert_eq!(PersonalEntry::parse_str(s2, 0), Ok(r2));
    assert_eq!(PersonalEntry::parse_str(s3, 0), Ok(r3));
    assert_eq!(PersonalEntry::parse_str(s4, 0), Ok(r4));
}

#[test]
fn test_update_personal() {
    let personal_str = r"
        abcd po:verb
        efgh st:something
        *ijkl
        mnop
        qrst
        uvwx st:something
        *yz12 po:verb
        3456
    ";

    let mut d = Dictionary::new();
    d.parse_update_personal(personal_str, &[]).unwrap();
    assert!(d.wordlist.0.get("abcd").is_some());
    assert!(d.wordlist.0.get("efgh").is_some());
    assert!(d.wordlist.0.get("ijkl").is_none());

    dbg!(&d.wordlist.0);
    dbg!(&d.wordlist.0.len());
    dbg!(&d.wordlist_nosuggest.0);
    dbg!(&d.wordlist_nosuggest.0.len());
    dbg!(&d.wordlist_forbidden.0);
    dbg!(&d.wordlist_forbidden.0.len());
    dbg!(&d.sources);
    dbg!(&d.sources.len());
    dbg!(&d.stems);
    dbg!(&d.stems.len());
}

#[test]
fn test_builder() {
    use std::fs;

    let aff_content = fs::read_to_string("tests/files/w1_eng_short.aff").unwrap();
    let dic_content = fs::read_to_string("tests/files/w1_eng_short.dic").unwrap();
    let dict = DictBuilder::new()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .unwrap();

    assert_eq!(dict.check("reptiles pillow bananas"), true);
    assert_eq!(dict.check("pine missssspelled"), false);
}
