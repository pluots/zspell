//! Implementation for a store rule

use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

use regex::Regex;

use crate::affix::{ParsedConfig, RuleType};
use crate::helpers::{compile_re_pattern, ReWrapper};
use crate::morph::MorphInfo;
use crate::parser_affix::ParsedRuleGroup;

/// A single rule that contains the following:
///
/// - Type of rule (prefix or suffix)
/// - Characters to strip, if any
/// - Condition to strip characters
/// - Associated morph info
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AfxRule {
    flag: u32,
    kind: RuleType,
    affix: String,
    can_combine: bool,
    strip: Option<String>,
    condition: Option<ReWrapper>,
    morph_info: Vec<Rc<MorphInfo>>,
}

impl AfxRule {
    ///
    /// # Panics
    ///
    /// Panics if regex is provided invalid
    #[cfg(test)]
    pub fn new(
        flag: u32,
        kind: RuleType,
        affix: &str,
        can_combine: bool,
        strip: Option<&str>,
        condition: Option<&str>,
        morph_info: Vec<Rc<MorphInfo>>,
    ) -> Self {
        Self {
            flag,
            kind,
            affix: affix.to_string(),
            can_combine,
            strip: strip.map(|s| s.to_owned()),
            condition: condition.map(|s| ReWrapper::new(s).unwrap()),
            morph_info,
        }
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
    pub fn apply_pattern(&self, s: &str) -> Option<String> {
        // No return if condition doesn't match
        if !self.check_condition(s) {
            return None;
        }

        let mut working = s;

        match self.kind {
            RuleType::Prefix => {
                // If stripping chars exist, strip them from the prefix
                if let Some(sc) = &self.strip {
                    working = working.strip_prefix(sc.as_str()).unwrap_or(working);
                }

                let mut w_s = self.affix.clone();
                w_s.push_str(working);
                Some(w_s)
            }
            RuleType::Suffix => {
                // Same logic as above
                if let Some(sc) = &self.strip {
                    working = working.strip_suffix(sc.as_str()).unwrap_or(working);
                }

                let mut w_s = working.to_owned();
                w_s.push_str(&self.affix);
                Some(w_s)
            }
        }
    }

    pub fn apply_if_flag_matches(&self, s: &str, flag: u32) -> Option<String> {
        if flag != self.flag {
            None
        } else {
            self.apply_pattern(s)
        }
    }

    // /// Apply a rule twice; this rule must be a prefix and other must be a suffix
    // pub fn apply_two(&self, other: &Self) -> Option<String> {
    //     if !(self.can_combine && other.can_combine) ||
    //         self.kind == RuleType::Suffix || other.kind == RuleType::Prefix
    //     {
    //         return Err(())
    //     }
    // }

    /// Take a ParsedGroup and turn it into a vector of `AfxRule`
    ///
    /// NOTE: returns a vec reference and `Self`'s morph vec will be empty!
    /// Needs construction wherever the Rc target is
    // PERF: bench with & without vec reference instead of output
    pub fn from_group(cfg: &ParsedConfig, group: ParsedRuleGroup) -> Vec<(Self, Vec<MorphInfo>)> {
        let flag = cfg.convert_flag(&group.flag).unwrap();
        let mut ret = Vec::with_capacity(group.rules.len());
        for rule in group.rules {
            ret.push((
                Self {
                    flag,
                    kind: group.kind,
                    affix: rule.affix,
                    can_combine: group.can_combine,
                    strip: rule.strip,
                    condition: rule.condition,
                    morph_info: Vec::with_capacity(rule.morph_info.len()),
                },
                rule.morph_info,
            ))
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

    /// Helper for testing
    #[cfg(test)]
    pub fn set_re_pattern(&mut self, condition: &str, kind: RuleType) -> Result<(), regex::Error> {
        self.condition = compile_re_pattern(condition, kind)?;
        Ok(())
    }
}
