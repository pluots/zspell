use pretty_assertions::assert_eq;

use super::*;

#[test]
fn test_dict_entry_ok() {
    let f1 = FlagType::Utf8;
    let f2 = FlagType::Ascii;
    let f3 = FlagType::Long;

    let s_0f0m_1 = "abcd";
    let s_0f0m_2 = "abcd # comment";
    let s_4f0m_1 = "abcd/ABCD";
    let s_4f0m_2 = "abcd/ABCD # comment";
    let s_4f2m_1 = "abcd/ABCD ip:m1 tp:m2";
    let s_4f2m_2 = "abcd/ABCD ip:m1 tp:m2 # comment";
    let s_0f2m_1 = "abcd ip:m1 tp:m2";
    let s_0f2m_2 = "abcd ip:m1 tp:m2 # comment";

    // No flags
    let r_0f0m = DictEntry::new("abcd", &[], &[]);

    // All flags
    let r_4f0m = DictEntry::new(
        "abcd",
        &[
            Flag::new_ascii(b'A'),
            Flag::new_ascii(b'B'),
            Flag::new_ascii(b'C'),
            Flag::new_ascii(b'D'),
        ],
        &[],
    );

    let r_2f0m = DictEntry::new("abcd", &[Flag::new_long("AB"), Flag::new_long("CD")], &[]);

    // All flags plus morph info
    let r_4f2m = DictEntry::new(
        "abcd",
        &[
            Flag::new_ascii(b'A'),
            Flag::new_ascii(b'B'),
            Flag::new_ascii(b'C'),
            Flag::new_ascii(b'D'),
        ],
        &[
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
    );

    let r_2f2m = DictEntry::new(
        "abcd",
        &[Flag::new_long("AB"), Flag::new_long("CD")],
        &[
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
    );

    // No flags, including morph info
    let r_0f2m = DictEntry::new(
        "abcd",
        &[],
        &[
            MorphInfo::InflecPfx("m1".into()),
            MorphInfo::TermPfx("m2".into()),
        ],
    );

    assert_eq!(DictEntry::parse_single(s_0f0m_1, f1, 0), Ok(r_0f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f0m_2, f1, 0), Ok(r_0f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f0m_1, f1, 0), Ok(r_4f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f0m_2, f1, 0), Ok(r_4f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f2m_1, f1, 0), Ok(r_4f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f2m_2, f1, 0), Ok(r_4f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f2m_1, f1, 0), Ok(r_0f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f2m_2, f1, 0), Ok(r_0f2m.clone()));

    assert_eq!(DictEntry::parse_single(s_0f0m_1, f2, 0), Ok(r_0f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f0m_2, f2, 0), Ok(r_0f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f0m_1, f2, 0), Ok(r_4f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f0m_2, f2, 0), Ok(r_4f0m));
    assert_eq!(DictEntry::parse_single(s_4f2m_1, f2, 0), Ok(r_4f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f2m_1, f2, 0), Ok(r_4f2m));
    assert_eq!(DictEntry::parse_single(s_0f2m_2, f2, 0), Ok(r_0f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f2m_2, f2, 0), Ok(r_0f2m.clone()));

    assert_eq!(DictEntry::parse_single(s_0f0m_1, f3, 0), Ok(r_0f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f0m_2, f3, 0), Ok(r_0f0m));
    assert_eq!(DictEntry::parse_single(s_4f0m_1, f3, 0), Ok(r_2f0m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f0m_2, f3, 0), Ok(r_2f0m));
    assert_eq!(DictEntry::parse_single(s_4f2m_1, f3, 0), Ok(r_2f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_4f2m_1, f3, 0), Ok(r_2f2m));
    assert_eq!(DictEntry::parse_single(s_0f2m_1, f3, 0), Ok(r_0f2m.clone()));
    assert_eq!(DictEntry::parse_single(s_0f2m_2, f3, 0), Ok(r_0f2m));
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
