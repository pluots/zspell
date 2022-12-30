use std::hash::Hash;

use regex::Regex;

use crate::error::ParseErrorKind;
use crate::helpers::{compile_re_pattern, ReWrapper};
use crate::morph::MorphInfo;
use crate::parser_affix::RuleType;
use crate::Error;

/// A simple prefix or suffix rule
///
/// This struct represents a prefix or suffix option that may be applied to any
/// base word. It contains multiple possible rule definitions that describe how
/// to apply the rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedRuleGroup {
    /// Character identifier for this specific affix, usually any uppercase
    /// letter
    pub(crate) flag: String,
    /// Prefix or suffix
    pub(crate) kind: RuleType,
    /// Whether or not this can be combined with the opposite affix
    pub(crate) can_combine: bool,
    /// Actual rules for replacing
    pub(crate) rules: Vec<ParsedRule>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedRule {
    /// Affix to be added
    pub(crate) affix: String,
    /// Characters to remove from the beginning or end
    pub(crate) strip: Option<String>,
    /// Regex-based rule for when this rule is true. `None` indicates `.`, i.e.,
    /// always true
    pub(crate) condition: Option<ReWrapper>,
    /// Morphological information
    pub(crate) morph_info: Vec<MorphInfo>,
}

impl ParsedRule {
    pub(crate) fn new(
        kind: RuleType,
        affix: &str,
        strip: Option<&str>,
        condition: Option<&str>,
        morph_info: Vec<MorphInfo>,
    ) -> Result<Self, Error> {
        let cond_re = match condition {
            Some(c) => compile_re_pattern(c, kind)?,
            None => None,
        };

        Ok(Self {
            strip: strip.map(ToOwned::to_owned),
            affix: affix.to_owned(),
            condition: cond_re,
            morph_info,
        })
    }

    /// Same as `new` but don't modify the regex string
    pub(crate) fn new_raw_re(
        kind: RuleType,
        affix: &str,
        strip: Option<&str>,
        condition: Option<&str>,
        morph_info: Vec<MorphInfo>,
    ) -> Result<Self, Error> {
        let cond_re = match condition {
            Some(c) => Some(ReWrapper::new(c)?),
            None => None,
        };

        Ok(Self {
            strip: strip.map(ToOwned::to_owned),
            affix: affix.to_owned(),
            condition: cond_re,
            morph_info,
        })
    }

    /// Create from the information we have available during parse
    pub(crate) fn new_parse(
        kind: RuleType,
        affix: &str,
        strip: &str,
        condition: &str,
        morph_info: Vec<MorphInfo>,
    ) -> Result<Self, ParseErrorKind> {
        let cond_re = compile_re_pattern(condition, kind)?;
        let strip_chars = if strip == "0" {
            None
        } else {
            Some(strip.to_owned())
        };

        Ok(Self {
            strip: strip_chars,
            affix: affix.to_owned(),
            condition: cond_re,
            morph_info,
        })
    }
}
