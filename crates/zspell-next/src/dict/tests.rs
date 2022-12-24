//! Tests for a dict file

use super::parser::DictEntry;
use super::*;
use crate::affix::types::MorphInfo;

#[test]
fn test_dict_entry_ok() {
    let s1 = "abcd";
    let s2 = "abcd # comment";
    let s3 = "abcd/ABC";
    let s4 = "abcd/ABC # comment";
    let s5 = "abcd/ABC ip:m1 tp:m2";
    let s6 = "abcd/ABC ip:m1 tp:m2 # comment";
    let s7 = "abcd ip:m1 tp:m2";
    let s8 = "abcd ip:m1 tp:m2 # comment";

    let r1 = DictEntry::new("abcd".to_owned(), Vec::new(), Vec::new());
    let r2 = DictEntry::new(
        "abcd".to_owned(),
        vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
        Vec::new(),
    );
    let r3 = DictEntry::new(
        "abcd".to_owned(),
        vec!["A".to_owned(), "B".to_owned(), "C".to_owned()],
        vec![
            MorphInfo::InflecPfx("m1".to_owned()),
            MorphInfo::TermPfx("m2".to_owned()),
        ],
    );
    let r4 = DictEntry::new(
        "abcd".to_owned(),
        Vec::new(),
        vec![
            MorphInfo::InflecPfx("m1".to_owned()),
            MorphInfo::TermPfx("m2".to_owned()),
        ],
    );

    assert_eq!(DictEntry::parse_str(s1, 0), Ok(r1.clone()));
    assert_eq!(DictEntry::parse_str(s2, 0), Ok(r1));
    assert_eq!(DictEntry::parse_str(s3, 0), Ok(r2.clone()));
    assert_eq!(DictEntry::parse_str(s4, 0), Ok(r2));
    assert_eq!(DictEntry::parse_str(s5, 0), Ok(r3.clone()));
    assert_eq!(DictEntry::parse_str(s6, 0), Ok(r3));
    assert_eq!(DictEntry::parse_str(s7, 0), Ok(r4.clone()));
    assert_eq!(DictEntry::parse_str(s8, 0), Ok(r4));
}
