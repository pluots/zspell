use pretty_assertions::assert_eq;

// use super::parser::DictEntry;
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

    let r1 = DictEntry::new("abcd", &[], Vec::new());
    let r2 = DictEntry::new("abcd", &['A' as u32, 'B' as u32, 'C' as u32], Vec::new());
    let r3 = DictEntry::new(
        "abcd",
        &['A' as u32, 'B' as u32, 'C' as u32],
        vec![
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
    );
    let r4 = DictEntry::new(
        "abcd",
        &[],
        vec![
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
    );

    assert_eq!(DictEntry::parse_single(s1, f1, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_single(s2, f1, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_single(s3, f1, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_single(s4, f1, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_single(s5, f1, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_single(s6, f1, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_single(s7, f1, 0), Ok(r4.clone()));
    assert_eq!(DictEntry::parse_single(s8, f1, 0), Ok(r4.clone()));

    assert_eq!(DictEntry::parse_single(s1, f2, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_single(s2, f2, 0), Ok(r1));
    assert_eq!(DictEntry::parse_single(s3, f2, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_single(s4, f2, 0), Ok(r2));
    assert_eq!(DictEntry::parse_single(s5, f2, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_single(s6, f2, 0), Ok(r3));
    assert_eq!(DictEntry::parse_single(s7, f2, 0), Ok(r4.clone()));
    assert_eq!(DictEntry::parse_single(s8, f2, 0), Ok(r4));
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
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
        false,
    );

    assert_eq!(PersonalEntry::parse_single(s1), r1);
    assert_eq!(PersonalEntry::parse_single(s2), r2);
    assert_eq!(PersonalEntry::parse_single(s3), r3);
    assert_eq!(PersonalEntry::parse_single(s4), r4);
}
