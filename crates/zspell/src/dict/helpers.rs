use std::borrow::Borrow;
use std::rc::Rc;

use hashbrown::HashSet;

use super::rule::AfxRule;
use crate::affix::{FlagType, RuleType};
use crate::error::BuildError;

pub(super) fn create_affixed_words<'a, I, D>(
    rules: I,
    stem: &str,
    flags: &[u32],
) -> Vec<(String, &'a AfxRule, Option<&'a AfxRule>)>
where
    I: IntoIterator<Item = &'a D>,
    D: Borrow<AfxRule> + 'a,
{
    // BENCH: new vs. with capacity (with cap flags.len()?)
    let mut ret: Vec<(String, &AfxRule, Option<&AfxRule>)> = Vec::new();

    if flags.into_iter().count() == 0 {
        return ret;
    }

    // Store words with prefixes that can also have suffixes
    let mut prefixed_words: Vec<(String, &AfxRule)> = Vec::new();
    let mut suffix_rules: Vec<&AfxRule> = Vec::new();

    // Loop through rules where the flag matches and there are new words to
    // create.
    rules
        .into_iter()
        // Use a fake `contains` because of `as_ref` (asm is about the same)
        .filter_map(|rule| {
            flags.iter().find_map(|flag| {
                rule.borrow()
                    .apply_if_flag_matches(stem, *flag)
                    .map(|newword| (rule, newword))
            })
        })
        .for_each(|(rule_b, newword)| {
            let rule = rule_b.borrow();
            if rule.can_combine() {
                // For rules that can combine: if a prefix, store the
                // word. If a suffix, store the rule. We'll go through
                // and cross match these
                if rule.is_pfx() {
                    prefixed_words.push((newword.clone(), rule));
                } else {
                    suffix_rules.push(rule);
                }
            }

            // Add the new word
            ret.push((newword, rule, None));
        });

    // Loop our prefixed words that allow suffixes
    let double_matches = prefixed_words.iter().flat_map(|(pfxword, pfxrule)| {
        // Collect suffix rules that match
        suffix_rules.iter().filter_map(|sfxrule| {
            sfxrule
                .apply_pattern(pfxword)
                .map(|newword| (newword, *pfxrule, Some(*sfxrule)))
        })
    });

    ret.extend(double_matches);

    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_create_words() {
        // Does not yet check the rules component
        fn map_tuples<'a>(
            tup: &'a Vec<(String, &'a AfxRule, Option<&'a AfxRule>)>,
        ) -> Vec<&'a str> {
            // Turn our output into a vector for easier comparison
            tup.iter().map(|t| t.0.as_str()).collect()
        }

        let rules = [
            AfxRule::new(1, RuleType::Prefix, "aa", false, None, None, Vec::new()),
            AfxRule::new(2, RuleType::Prefix, "aa", true, None, None, Vec::new()),
            AfxRule::new(3, RuleType::Suffix, "bb", true, None, None, Vec::new()),
            AfxRule::new(3, RuleType::Suffix, "cc", true, None, None, Vec::new()),
        ];

        assert_eq!(
            map_tuples(&create_affixed_words(&rules, "xxx", &['A' as u32])),
            vec!["aaxxx"]
        );
        assert_eq!(
            map_tuples(&create_affixed_words(&rules, "xxx", &['B' as u32])),
            vec!["xxxcc"]
        );
        assert_eq!(
            map_tuples(&create_affixed_words(
                &rules,
                "xxx",
                &['A' as u32, 'B' as u32]
            )),
            vec!["aaxxx", "xxxcc", "aaxxxcc",]
        );
    }
}
