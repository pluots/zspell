use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use super::rule::AfxRule;
use super::WordList;
use crate::dict::types::{Meta, Source};

// pub(super) fn analyze_flags

pub(super) fn create_affixed_word_map(
    prefix_rules: &[&Arc<AfxRule>],
    suffix_rules: &[&Arc<AfxRule>],
    stem: &str,
    stem_rc: &Arc<str>,
    dest: &mut WordList,
) -> bool {
    if prefix_rules.is_empty() && suffix_rules.is_empty() {
        return false;
    }

    // Store words with prefixes that can also have suffixes
    let mut prefixed_words: Vec<(String, &Arc<AfxRule>, usize)> = Vec::new();
    let mut rule_found = false;

    for &rule in prefix_rules {
        for (idx, result) in rule.apply_patterns(stem) {
            let meta = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
            let meta_vec = dest.0.entry_ref(result.as_str()).or_insert_with(Vec::new);
            meta_vec.push(meta);
            rule_found = true;

            if rule.can_combine() {
                prefixed_words.push((result, rule, idx));
            }
        }
    }

    for &rule in suffix_rules {
        for (_idx, result) in rule.apply_patterns(stem) {
            let meta = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
            let meta_vec = dest.0.entry_ref(result.as_str()).or_insert_with(Vec::new);
            meta_vec.push(meta);
            rule_found = true;

            if rule.can_combine() {
                // Find words where there's both a prefix and suffix applicable
                let words_iter = prefixed_words
                    .iter()
                    .flat_map(|(tmp_res, pfx_rule, idx_pfx)| {
                        rule.apply_patterns(tmp_res)
                            .map(move |(idx_sfx, newword)| (newword, pfx_rule, idx_pfx, idx_sfx))
                    });

                for (newword, &pfx_rule, _idx_pfx, _idx_sfx) in words_iter {
                    let meta_vec = dest.0.entry_ref(newword.as_str()).or_insert_with(Vec::new);
                    let meta1 = Meta::new(stem_rc.clone(), Source::Affix(rule.clone()));
                    let meta2 = Meta::new(stem_rc.clone(), Source::Affix(pfx_rule.clone()));
                    meta_vec.push(meta1);
                    meta_vec.push(meta2);
                }
            }
        }
    }

    rule_found
}

/// Segment words by unicode boundaries.
pub fn word_splitter(s: &str) -> impl Iterator<Item = (usize, &str)> {
    s.split_word_bound_indices()
        .filter(|split| split.1.chars().all(|c| c.is_alphanumeric() || c == '-'))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::affix::RuleType;

    #[test]
    fn test_create_words() {
        let rul1 = Arc::new(AfxRule::new(
            RuleType::Prefix,
            &["aa"],
            &["."],
            false,
            None,
            None,
        ));
        let rul2 = Arc::new(AfxRule::new(
            RuleType::Prefix,
            &["bb"],
            &["."],
            true,
            None,
            None,
        ));
        let rul3 = Arc::new(AfxRule::new(
            RuleType::Suffix,
            &["cc", "dd"],
            &["x", "[^x]"],
            true,
            None,
            None,
        ));

        let conditions = [
            ("xxx", &[&rul1][..], &[][..], &["aaxxx"][..]),
            ("xxx", &[&rul2][..], &[][..], &["bbxxx"][..]),
            ("xxx", &[][..], &[&rul3][..], &["xxxcc"][..]),
            ("yyy", &[][..], &[&rul3][..], &["yyydd"][..]),
            ("xxx", &[&rul1, &rul2][..], &[][..], &["aaxxx", "bbxxx"][..]),
            ("xxx", &[&rul1][..], &[&rul3][..], &["aaxxx", "xxxcc"][..]),
            ("yyy", &[&rul1][..], &[&rul3][..], &["aayyy", "yyydd"][..]),
            (
                "xxx",
                &[&rul2][..],
                &[&rul3][..],
                &["bbxxx", "xxxcc", "bbxxxcc"][..],
            ),
            (
                "yyy",
                &[&rul2][..],
                &[&rul3][..],
                &["bbyyy", "yyydd", "bbyyydd"][..],
            ),
            (
                "xxx",
                &[&rul1, &rul2][..],
                &[&rul3][..],
                &["aaxxx", "bbxxx", "xxxcc", "bbxxxcc"][..],
            ),
            (
                "yyy",
                &[&rul1, &rul2][..],
                &[&rul3][..],
                &["aayyy", "bbyyy", "yyydd", "bbyyydd"][..],
            ),
        ];

        for (i, (word, pfxs, sfxs, expected_slice)) in conditions.iter().enumerate() {
            let mut dest = WordList::new();
            let stem_rc = Arc::from(*word);
            create_affixed_word_map(pfxs, sfxs, &stem_rc, &stem_rc, &mut dest);

            let tmp: Vec<(Box<str>, _)> = dest.0.into_iter().collect();
            let mut result: Vec<_> = tmp.iter().map(|(s, _)| s.as_ref()).collect();
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
        let s = "the quick brown.     Fox Jum-ped --\t where? 'over' (the) very--lazy dog";
        let _: Vec<_> = dbg!(word_splitter(s).collect());
        let _: Vec<_> = dbg!(s.split_word_bound_indices().collect());
        // FIXME: do something with these
    }
}

// TODO: evaluate this for hyphenation
// mod peek_map {
//     use std::iter::Peekable;
//     use unicode_segmentation::UnicodeSegmentation;

//     pub struct PeekMap<I: Iterator, F>(Peekable<I>, F);

//     pub fn peek_map<R, I: Iterator, F: FnMut(I::Item, Option<&I::Item>) -> R>(
//         it: Peekable<I>,
//         f: F,
//     ) -> PeekMap<I, F> {
//         PeekMap(it, f)
//     }

//     impl<R, I: Iterator, F: FnMut(I::Item, Option<&I::Item>) -> R> Iterator for PeekMap<I, F> {
//         type Item = R;
//         fn next(&mut self) -> Option<R> {
//             let x = self.0.next()?;
//             Some((self.1)(x, self.0.peek()))
//         }
//     }

//     #[cfg(test)]
//     mod tests {
//         use super::*;

//         #[test]
//         fn test_x() {
//             let s = "the quick brown.   Fox Jum-ped -- where? 'over' (the) very-lazy dog";

//             enum HyphenState {
//                 None,
//                 AwaitingHyphen(usize),
//                 AwaitingWord(usize)
//             }

//             let mut accum = HyphenState::None;

//             let v: Vec<_> = peek_map(s.split_word_bound_indices().peekable(),
//                 |(idx, w), next|{

//                 let c1 = w.chars().next().unwrap();
//                 if !(c1.is_alphanumeric() || c1 == '-') {
//                     accum = HyphenState::None;
//                     return None;
//                 }

//                 if let Some((nidx, nw)) = next {
//                     // If our next item is a hyphen, start accumulating
//                     if nw == "-" {
//                         accum = HyphenState::AwaitingHyphen(idx);
//                         return None;
//                     }
//                 }
//                 match accum {
//                     HyphenState::None => {
//                         // No upcoming hyphen? Just return our value
//                         Some((idx, w))
//                     },
//                     HyphenState::AwaitingHyphen(_) => {

//                     },
//                     HyphenState::AwaitingWord(_) => todo!(),
//                 }
//             }
//             ).collect();

//             dbg!(v);

//         }
//     }
// }
