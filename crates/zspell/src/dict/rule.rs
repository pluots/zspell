//! Implementation for a store rule

use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

use regex::Regex;

use crate::affix::{ParsedCfg, RuleType};
use crate::error::BuildError;
use crate::helpers::{compile_re_pattern, ReWrapper};
use crate::morph::MorphInfo;
use crate::parser_affix::ParsedRuleGroup;
use crate::Error;

/// A single rule group
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AfxRule {
    kind: RuleType,
    can_combine: bool,
    patterns: Vec<AfxRulePattern>,
}

impl AfxRule {
    /// Creates a rule with a single pattern
    #[cfg(test)]
    pub fn new(
        kind: RuleType,
        affixes: &[&str],
        patterns: &[&str],
        can_combine: bool,
        strip: Option<&str>,
        condition: Option<&str>,
    ) -> Self {
        let mut ret = Self {
            kind,
            can_combine,
            patterns: affixes
                .iter()
                .map(|afx| AfxRulePattern::new(afx, None))
                .collect(),
        };
        for (idx, pat) in patterns.iter().enumerate() {
            ret.patterns[idx].set_pattern(pat, kind);
        }
        ret
    }

    /// Take a [`ParsedGroup`] and turn it into a vector of `AfxRule`
    ///
    /// NOTE: returns a vec reference and `Self`'s morph vec will be empty!
    /// Needs construction wherever the Arc target is
    // PERF: bench with & without vec reference instead of output
    pub fn from_parsed_group(cfg: &ParsedCfg, group: &ParsedRuleGroup) -> Self {
        let mut ret = Self {
            kind: group.kind,
            can_combine: group.can_combine,
            patterns: Vec::with_capacity(group.rules.len()),
        };

        for rule in &group.rules {
            let mut morph_info: Vec<Arc<MorphInfo>> = rule
                .morph_info
                .iter()
                .map(|m| Arc::new(m.clone()))
                .collect();

            ret.patterns.push(AfxRulePattern {
                affix: rule.affix.clone(),
                condition: rule.condition.clone(),
                strip: rule.strip.clone(),
                morph_info,
            });
        }

        ret
    }

    pub fn is_pfx(&self) -> bool {
        self.kind == RuleType::Prefix
    }

    pub fn is_sfx(&self) -> bool {
        self.kind == RuleType::Prefix
    }

    pub fn can_combine(&self) -> bool {
        self.can_combine
    }

    /// Apply one of this rule's patterns, error if none apply
    pub fn apply_pattern(&self, stem: &str) -> Option<String> {
        self.patterns
            .iter()
            .find_map(|pat| pat.apply_pattern(stem, self.kind))
    }
}

/// A single rule
#[derive(Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct AfxRulePattern {
    affix: String,
    /// Condition to be met to apply this rule.
    condition: Option<ReWrapper>,
    /// Characters to strip
    strip: Option<String>,
    /// Associated morph info
    morph_info: Vec<Arc<MorphInfo>>,
}

impl AfxRulePattern {
    /// New with a specified affix, otherwise default values
    #[cfg(test)]
    pub fn new(afx: &str, strip: Option<&str>) -> Self {
        Self {
            affix: afx.to_owned(),
            condition: None,
            strip: strip.map(ToOwned::to_owned),
            morph_info: Vec::new(),
        }
    }

    /// Helper for testing, sets the condition based on a kind
    #[cfg(test)]
    pub fn set_pattern(&mut self, condition: &str, kind: RuleType) -> Result<(), regex::Error> {
        self.condition = compile_re_pattern(condition, kind)?;
        Ok(())
    }

    /// Check whether a condition is applicable
    #[allow(clippy::option_if_let_else)]
    pub fn check_condition(&self, s: &str) -> bool {
        match &self.condition {
            Some(re) => re.is_match(s),
            None => true,
        }
    }

    // Verify the match condition and apply this rule
    #[allow(clippy::option_if_let_else)]
    fn apply_pattern(&self, s: &str, kind: RuleType) -> Option<String> {
        // No return if condition doesn't match
        if !self.check_condition(s) {
            return None;
        }

        match kind {
            RuleType::Prefix => {
                // If stripping chars exist, strip them from the prefix
                let mut working = self.affix.clone();

                if let Some(sc) = &self.strip {
                    working.push_str(s.strip_prefix(sc.as_str()).unwrap_or(s));
                } else {
                    working.push_str(s);
                }
                working.shrink_to_fit();
                Some(working)
            }
            RuleType::Suffix => {
                // Same logic as above
                let mut working = if let Some(sc) = &self.strip {
                    s.strip_suffix(sc.as_str()).unwrap_or(s).to_owned()
                } else {
                    s.to_owned()
                };
                working.push_str(&self.affix);
                working.shrink_to_fit();
                Some(working)
            }
        }
    }
}

#[cfg(test)]
mod tests;
