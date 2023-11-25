use super::*;
use crate::affix::RuleType::{self, Prefix, Suffix};

#[test]
fn test_check_condition() {
    let mut kind = RuleType::Suffix;
    let mut rule = AfxRulePattern::default();
    rule.set_pattern("[^aeiou]y", kind).unwrap();

    // General tests, including with pattern in the middle
    assert!(rule.check_condition("xxxy"));
    assert!(!rule.check_condition("xxxay"));
    assert!(!rule.check_condition("xxxyxx"));

    // Test with prefix
    kind = RuleType::Prefix;
    rule.set_pattern("y[^aeiou]", kind).unwrap();
    assert!(rule.check_condition("yxxx"));
    assert!(!rule.check_condition("yaxxx"));
    assert!(!rule.check_condition("xxxyxxx"));

    // Test other real rules
    kind = RuleType::Suffix;
    rule.set_pattern("[sxzh]", kind).unwrap();
    assert!(rule.check_condition("access"));
    assert!(rule.check_condition("abyss"));
    assert!(!rule.check_condition("accomplishment"));
    assert!(rule.check_condition("mmms"));
    assert!(!rule.check_condition("mmsmm"));

    // Check with default condition
    rule.set_pattern(".", kind).unwrap();
    assert!(rule.check_condition("xxx"));
}

// affix, strip, condition, kind, input, output
const RULE_PATTERNS: &[(&str, Option<&str>, &str, RuleType, &str, &str)] = &[
    ("zzz", Some("y"), "[^aeiou]y", Suffix, "xxxy", "xxxzzz"),
    ("zzz", Some("y"), "y[^aeiou]", Prefix, "yxxx", "zzzxxx"),
    ("zzz", None, ".", Suffix, "xxx", "xxxzzz"),
];

#[test]
fn test_apply_pattern() {
    for rule_pat in RULE_PATTERNS {
        let (afx, strip, cond, kind, input, output) = rule_pat;
        let mut rule = AfxRulePattern::new(afx, *strip);
        rule.set_pattern(cond, *kind).unwrap();

        assert_eq!(
            rule.apply_pattern(input, *kind),
            Some((*output).into()),
            "testing {rule_pat:?}"
        );
    }
}

#[test]
fn test_strip_pattern() {
    for rule_pat in RULE_PATTERNS {
        let (afx, strip, cond, kind, input, output) = rule_pat;
        let mut rule = AfxRulePattern::new(afx, *strip);
        rule.set_pattern(cond, *kind).unwrap();

        assert_eq!(
            rule.strip_pattern(output, *kind),
            Some((*input).into()),
            "testing {rule_pat:?}"
        );
    }
}

// #[test]
// fn test_rule_group_apply_pattern() {
//     let kind = RuleType::Suffix;
//     let rules= vec![
//         AfxRule::new(0, kind, "iness",false, Some("y"), Some("[^aeiou]y"), Vec::new()),
//         AfxRule::new(0, kind, "ness",false, None, Some("[aeiou]y"), Vec::new()),
//         AfxRule::new(0, kind, "ness",false, None, Some("[^y]"), Vec::new()),
//     ];

//     assert_eq!(group.apply_pattern("blurry").unwrap(), "blurriness");
//     assert_eq!(group.apply_pattern("coy").unwrap(), "coyness");
//     assert_eq!(group.apply_pattern("acute").unwrap(), "acuteness");
// }
