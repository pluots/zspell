use std::borrow::Borrow;
use std::fmt::Debug;
use std::rc::Rc;

use hashbrown::HashSet;
use unicode_segmentation::UnicodeSegmentation;

use super::rule::AfxRule;
use super::{FlagValue, WordList};
use crate::affix::{FlagType, RuleType};
use crate::dict::types::{Meta, Source};
use crate::error::BuildError;
use crate::Error;

// pub(super) fn analyze_flags

pub(super) fn create_affixed_word_map(
    prefix_rules: &[&Rc<AfxRule>],
    suffix_rules: &[&Rc<AfxRule>],
    stem: &str,
    stem_rc: &Rc<String>,
    dest: &mut WordList,
) -> Result<(), ()> {
    if prefix_rules.is_empty() && suffix_rules.is_empty() {
        return Ok(());
    }

    // Store words with prefixes that can also have suffixes
    let mut prefixed_words: Vec<(String, &Rc<AfxRule>)> = Vec::new();

    for &rule in prefix_rules.iter() {
        let result = rule.apply_pattern(stem).ok_or(())?;
        let meta = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
        let meta_vec = dest.0.entry_ref(&result).or_insert_with(Vec::new);
        meta_vec.push(meta);

        if rule.can_combine() {
            prefixed_words.push((result, rule));
        }
    }

    for &rule in suffix_rules.iter() {
        let result = rule.apply_pattern(stem).ok_or(())?;
        let meta = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
        let meta_vec = dest.0.entry_ref(&result).or_insert_with(Vec::new);
        meta_vec.push(meta);

        if rule.can_combine() {
            let words_iter = prefixed_words.iter().filter_map(|(tmp_res, pfx_rule)| {
                rule.apply_pattern(tmp_res)
                    .map(|newword| (newword, pfx_rule))
            });

            for (newword, &pfx_rule) in words_iter {
                let meta_vec = dest.0.entry_ref(&newword).or_insert_with(Vec::new);
                let meta1 = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
                let meta2 = Meta::new(stem_rc.clone(), Source::Affix(pfx_rule.clone()));
                meta_vec.push(meta1);
                meta_vec.push(meta2);
            }
        }
    }

    Ok(())
}

pub fn word_splitter(s: &str) -> impl Iterator<Item = (usize, &str)> {
    s.split_word_bound_indices()
        .filter(|split| split.1.chars().all(|c| c.is_alphanumeric() || c == '-'))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::dict::rule::AfxRulePattern;

    #[test]
    fn test_create_words() {
        let rul1 = Rc::new(AfxRule::new(RuleType::Prefix, &["aa"], false, None, None));
        let rul2 = Rc::new(AfxRule::new(RuleType::Prefix, &["bb"], true, None, None));
        let rul3 = Rc::new(AfxRule::new(
            RuleType::Suffix,
            &["cc", "dd"],
            true,
            None,
            None,
        ));

        let conditions = [
            (&[&rul1][..], &[][..], &["aaxxx"][..]),
            (&[&rul2][..], &[][..], &["bbxxx"][..]),
            (&[][..], &[&rul3][..], &["xxxcc", "xxxdd"][..]),
            (&[&rul1, &rul2][..], &[][..], &["aaxxx", "bbxxx"][..]),
            (&[&rul1][..], &[&rul3][..], &["aaxxx", "xxxcc", "xxxdd"][..]),
            (
                &[&rul2][..],
                &[&rul3][..],
                &["bbxxx", "xxxcc", "xxxdd", "bbxxxcc", "bbxxxdd"][..],
            ),
            (
                &[&rul1, &rul2][..],
                &[&rul3][..],
                &["aaxxx", "bbxxx", "xxxcc", "xxxdd", "bbxxxcc", "bbxxxdd"][..],
            ),
        ];

        for (i, (pfxs, sfxs, expected_slice)) in conditions.iter().enumerate() {
            let mut dest = WordList::new();
            let stem_rc = Rc::new("xxx".to_string());
            create_affixed_word_map(pfxs, sfxs, &stem_rc, &stem_rc, &mut dest);

            let mut tmp: Vec<(String, _)> = dest.0.into_iter().collect();
            let mut result: Vec<_> = tmp.iter().map(|(s, _)| s.as_str()).collect();
            let mut expected: Vec<_> = (*expected_slice).to_owned();
            result.sort_unstable();
            expected.sort_unstable();

            assert_eq!(
                result, expected,
                "testing index {i} with prefixes: {pfxs:#?}\nand suffixes: {sfxs:#?}"
            );
        }
    }

    #[test]
    fn test_word_splitter() {
        word_splitter(r#"the quick brown. Fox Jum-ped "over" (the) lazy dog"#);
    }
}
