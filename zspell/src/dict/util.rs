use std::sync::Arc;

use unicode_segmentation::UnicodeSegmentation;

use super::rule::AfxRule;
use super::WordList;
use crate::dict::meta::{Meta, Source};

/// A rule that may be combined with another
type PossibleCombination<'a> = (String, &'a Arc<AfxRule>, usize);

/// For a given stem, find all prefix and suffix rules that can apply, and store them
/// to a wordlist.
///
/// Also finds words
#[allow(clippy::similar_names)] // thinks pfx and sfx are too similar
pub(super) fn create_affixed_word_map(
    pfx_rules: &[&Arc<AfxRule>],
    sfx_rules: &[&Arc<AfxRule>],
    stem: &Arc<str>,
    dest: &mut WordList,
) -> bool {
    if pfx_rules.is_empty() && sfx_rules.is_empty() {
        return false;
    }

    // Store words with prefixes that can also have suffixes
    let mut pfxd_maybe_sfx: Vec<PossibleCombination> = Vec::new();
    let mut rule_found = false;

    for &pfx_rule in pfx_rules {
        // Locate matching prefix rules
        for (pat_idx, prefixed) in pfx_rule.apply_patterns(stem) {
            store_applied_pattern(stem, pfx_rule, pat_idx, &prefixed, dest);

            rule_found = true;

            // Save rules that can have a prefix and a suffix
            if pfx_rule.can_combine() {
                pfxd_maybe_sfx.push((prefixed, pfx_rule, pat_idx));
            }
        }
    }

    for &sfx_rule in sfx_rules {
        // Locate matching prefix rules
        for (pat_idx, suffixed) in sfx_rule.apply_patterns(stem) {
            store_applied_pattern(stem, sfx_rule, pat_idx, &suffixed, dest);
            rule_found = true;

            if sfx_rule.can_combine() {
                apply_combo_words(stem, &pfxd_maybe_sfx, sfx_rule, dest);
            }
        }
    }

    rule_found
}

/// Create meta and store an applied pattern to a wordlist
fn store_applied_pattern(
    stem_arc: &Arc<str>, // stem word
    rule: &Arc<AfxRule>, // rule that was applied
    pat_idx: usize,      // index of the relevant pattern within the rule
    affixed: &str,       // affixed (created) word
    dest: &mut WordList, // store the result here
) {
    // Create metadata for this application
    let meta = Meta::new(Arc::clone(stem_arc), Source::new_affix(rule, pat_idx));

    // Add this entry to the wordlist or update an existing one
    let meta_vec = dest.0.entry_ref(affixed).or_default();
    meta_vec.push(meta);
}

/// Given a list of words that are eligible for combinations, check if a rule applies. If
/// so, save it to the word list
fn apply_combo_words(
    stem_arc: &Arc<str>,
    pfxd_maybe_sfx: &[PossibleCombination],
    rule: &Arc<AfxRule>,
    dest: &mut WordList,
) {
    for (prefixed, pfx_rule, pfx_idx) in pfxd_maybe_sfx {
        for (sfx_idx, new_word) in rule.apply_patterns(prefixed) {
            let meta_vec = dest.0.entry_ref(new_word.as_str()).or_insert_with(Vec::new);

            let meta1 = Meta::new(stem_arc.clone(), Source::new_affix(pfx_rule, *pfx_idx));
            let meta2 = Meta::new(stem_arc.clone(), Source::new_affix(rule, sfx_idx));
            meta_vec.push(meta1);
            meta_vec.push(meta2);
        }
    }
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
            create_affixed_word_map(pfxs, sfxs, &stem_rc, &mut dest);

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
