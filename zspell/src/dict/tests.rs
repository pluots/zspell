//! Tests for a dict file

use std::fs;

use indoc::indoc;
use pretty_assertions::assert_eq;
use test_util::workspace_root;

use super::*;

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

    let mut d = Dictionary::new(ParsedCfg::default()).unwrap();
    d.parse_update_personal(personal_str, &[]).unwrap();
    assert!(d.wordlist.0.contains_key("abcd"));
    assert!(d.wordlist.0.contains_key("efgh"));
    assert!(!d.wordlist.0.contains_key("ijkl"));
    assert!(d.wordlist_forbidden.0.contains_key("ijkl"));
    assert!(d.check("abcd"));
    assert!(d.check("uvwx"));
    assert!(!d.check("ijkl"));

    let entry = d.entry("efgh");
    let stems: Vec<_> = entry.stems().unwrap().collect();
    assert_eq!(stems, vec!["efgh", "something"]);
}

#[test]
#[cfg(not(miri))] // slow!
fn test_builder() {
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

#[test]
fn test_builder_large_file() {
    let mut aff_path = workspace_root();
    aff_path.push("dictionaries");
    let mut dic_path = aff_path.clone();
    aff_path.push("en_US.aff");
    dic_path.push("en_US.dic");

    let Ok(aff_content) = fs::read_to_string(aff_path) else {
        eprintln!("skipping large test flies; not found");
        return;
    };

    let dic_content = fs::read_to_string(dic_path).unwrap();
    let dict = DictBuilder::new()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .unwrap();

    assert_eq!(dict.check("reptiles pillow bananas"), true);
    assert_eq!(dict.check("pine missssspelled"), false);
}

// Test how data is inserted
#[test]
fn test_morph() {
    use crate::DictBuilder;

    let dict_str = "drink/X po:verb";
    let aff_str = indoc! {"
        SFX X Y 1
        SFX X 0 able . ds:able
    "};

    let d = DictBuilder::new()
        .dict_str(dict_str)
        .config_str(aff_str)
        .build()
        .unwrap();

    let meta = d.wordlist.0.get("drinkable").unwrap();
    assert_eq!(meta[0].stem(), "drink");
    assert_eq!(meta[1].stem(), "drink");
    assert!(matches!(
        meta[0].source(),
        Source::Affix {
            rule: _,
            pat_idx: 0
        }
    ));

    let Source::Dict(mvec) = meta[1].source() else {
        panic!()
    };

    let po = MorphInfo::Part(crate::PartOfSpeech::Verb);
    assert_eq!(mvec.as_ref(), [po.clone().into()]);

    let entry = d.entry("drinkable");
    let morph = entry.analyze().unwrap().collect::<Vec<_>>();
    assert_eq!(morph, [&MorphInfo::DerivSfx("able".into()), &po]);

    let stems = entry.stems().unwrap().collect::<Vec<_>>();
    assert_eq!(stems, ["drink"]);
    // assert_eq!(stems, ["drinkable", "drink"]);
}
