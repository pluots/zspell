use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use util::workspace_root;

use super::*;
use crate::affix::{PartOfSpeech, RuleType};
use crate::error::Span;

#[test]
fn test_line_splitter_none() {
    let s = "no key here # abcd";
    assert_eq!(line_splitter(s, "KEY"), None);
}

#[test]
fn test_line_splitter_some() {
    let s1 = "KEY key here\nnext line";
    let s2 = "KEY key here# comment";
    let s3 = "KEY key here\rnext line";
    let s4 = "# comment here\n#next line";
    assert_eq!(line_splitter(s1, "KEY"), Some(("key here", "\nnext line")));
    assert_eq!(line_splitter(s2, "KEY"), Some(("key here", "# comment")));
    assert_eq!(line_splitter(s3, "KEY"), Some(("key here", "\rnext line")));
    assert_eq!(
        line_splitter(s4, "#"),
        Some(("comment here", "\n#next line"))
    );
}

#[test]
fn test_line_key_parser_none() {
    let s = "no key here # abcd";
    assert_eq!(
        line_key_parser(s, "KEY", |_| Ok(AffixNode::Comment)),
        Ok(None)
    );
}

#[test]
fn test_line_key_parser_some() {
    let s = "KEY key here\nnext line";
    assert_eq!(
        line_key_parser(s, "KEY", |_| Ok(AffixNode::Comment)),
        Ok(Some((AffixNode::Comment, "\nnext line", 0)))
    );
}

#[test]
fn test_line_key_parser_err() {
    let s = "KEY key here\nnext line";
    let e = ParseError::new_nospan(ParseErrorKind::Boolean, "");
    assert_eq!(line_key_parser(s, "KEY", |_| Err(e.clone())), Err(e));
}

#[test]
fn test_line_key_parser() {
    let err = ParseError::new_nospan(ParseErrorKind::Boolean, "");
    let get_lang = |s: &str| {
        if s == "apple" {
            Ok(AffixNode::Language("apple".to_owned()))
        } else {
            Err(err.clone())
        }
    };

    let txt1 = "LANG apple";
    let txt2 = "LANG apple\nLANG banana";
    let txt3 = "LANG failure";

    assert_eq!(
        line_key_parser(txt1, "LANG", get_lang),
        Ok(Some((AffixNode::Language("apple".to_owned()), "", 0)))
    );
    assert_eq!(
        line_key_parser(txt2, "LANG", get_lang),
        Ok(Some((
            AffixNode::Language("apple".to_owned()),
            "\nLANG banana",
            0
        )))
    );
    assert_eq!(line_key_parser(txt3, "LANG", get_lang), Err(err));
}

#[test]
fn test_parse_neighbor_keys() {
    let s = "KEY abc|def|ghi # end";
    let res = parse_neighbor_keys(s);
    assert_eq!(
        res,
        Ok(Some((
            AffixNode::NeighborKeys(vec!["abc".to_owned(), "def".to_owned(), "ghi".to_owned()]),
            "# end",
            0
        )))
    );
}

#[test]
fn test_bool_parser_ok() {
    let s = "COMPLEXPREFIXES\nmore stuff";
    let res = parse_complex_prefixes(s);
    assert_eq!(
        res,
        Ok(Some((AffixNode::ComplexPrefixes, "\nmore stuff", 0)))
    );
}

#[test]
fn test_bool_parser_err() {
    let s = "COMPLEXPREFIXES unneeded things\nmore stuff";
    let res = parse_complex_prefixes(s);
    assert!(res.is_err());
}

#[test]
fn test_munch_newline_some() {
    let s1 = "    \nabc";
    let s2 = "\n";
    assert_eq!(munch_newline(s1), Ok(Some("abc")));
    assert_eq!(munch_newline(s2), Ok(Some("")));
}

#[test]
fn test_munch_newline_none() {
    let s = "    ";
    assert_eq!(munch_newline(s), Ok(None));
}

#[test]
fn test_munch_newline_cmt() {
    let s = "  # abcd \nresid";
    assert_eq!(munch_newline(s), Ok(Some("resid")));
}

#[test]
fn test_munch_newline_err() {
    let s = "  abcd \nresid";
    assert!(munch_newline(s).is_err());
}

#[test]
fn test_table_parser_ok() {
    let s = "REP 3\nREP a b\nREP c d\nREP longer val";
    let expected = AffixNode::Replacement(vec![
        Conversion::new("a", "b", false),
        Conversion::new("c", "d", false),
        Conversion::new("longer", "val", false),
    ]);
    assert_eq!(parse_replacement(s), Ok(Some((expected, "", 3))));
}

#[test]
fn test_afx_table_parser_err() {
    // check line offset count
    let s = "PFX A N 2\nPFX A a b x .\nPFX A 0 c a";
    let res = parse_prefix(s);
    assert_eq!(res.unwrap_err().span().unwrap(), &Span::new(1, 0));
}

const SAMPLE_AFX_OK: &str = r#"
SET UTF-8
TRY abcd'
# comment
ICONV 2 # comment
ICONV a b # comment
ICONV ' "
NOSUGGEST X
ONLYINCOMPOUND C
WORDCHARS 01234
# comment
PFX A N 2
PFX A   0     ar   .    po:verb st:foot is:ay
PFX A   0     br   a

SFX B Y 2
SFX B   0     ar   .
SFX B   0     br   [^a]

REP 2
REP a b
REP abcd 123

PHONE 1
PHONE abcd 1234
"#;

#[test]
fn test_full_parse() {
    let expected = vec![
        AffixNode::Encoding(Encoding::Utf8),
        AffixNode::TryCharacters("abcd'".to_owned()),
        AffixNode::Comment,
        AffixNode::AfxInputConversion(vec![
            Conversion::new("a", "b", false),
            Conversion::new("'", "\"", false),
        ]),
        AffixNode::NoSuggestFlag("X".to_owned()),
        AffixNode::CompoundOnlyFlag("C".to_owned()),
        AffixNode::AfxWordChars("01234".to_owned()),
        AffixNode::Comment,
        AffixNode::Prefix(ParsedRuleGroup {
            flag: "A".to_owned(),
            kind: RuleType::Prefix,
            can_combine: false,
            rules: vec![
                ParsedRule::new_raw_re(
                    RuleType::Prefix,
                    "ar",
                    None,
                    None,
                    vec![
                        MorphInfo::Part(PartOfSpeech::Verb),
                        MorphInfo::Stem("foot".to_owned()),
                        MorphInfo::InflecSfx("ay".to_owned()),
                    ],
                )
                .unwrap(),
                ParsedRule::new_raw_re(RuleType::Prefix, "br", None, Some("^a.*$"), Vec::new())
                    .unwrap(),
            ],
        }),
        AffixNode::Suffix(ParsedRuleGroup {
            flag: "B".to_owned(),
            kind: RuleType::Suffix,
            can_combine: true,
            rules: vec![
                ParsedRule::new_raw_re(RuleType::Suffix, "ar", None, None, Vec::new()).unwrap(),
                ParsedRule::new_raw_re(RuleType::Suffix, "br", None, Some("^.*[^a]$"), Vec::new())
                    .unwrap(),
            ],
        }),
        AffixNode::Replacement(vec![
            Conversion::new("a", "b", false),
            Conversion::new("abcd", "123", false),
        ]),
        AffixNode::Phonetic(vec![Phonetic::new("abcd", "1234")]),
    ];

    assert_eq!(parse_affix(SAMPLE_AFX_OK), Ok(expected));
}

#[test]
fn test_large_file_parse() {
    let mut aff_path = workspace_root();
    aff_path.push("dictionaries");
    aff_path.push("en_US.aff");

    let Ok(aff_content) = fs::read_to_string(aff_path) else {
        eprintln!("skipping large test flies; not found");
        return;
    };

    assert!(parse_affix(&aff_content).is_ok());
}
