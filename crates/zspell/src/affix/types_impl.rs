//! Extension of the `types` module containing the messy impl blocks

use std::hash::Hash;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use super::types::{
    AffixRule, CompoundPattern, CompoundSyllable, Conversion, Encoding, FlagType, MorphInfo,
    PartOfSpeech, Phonetic, RuleGroup, RuleType,
};
use crate::error::{ParseError, ParseErrorType};
use crate::Error;

lazy_static! {
    static ref RE_COMPOUND_PATTERN: Regex = Regex::new(
        r"(?x)
        ^(?P<endchars>\w+)
        (?:/(?P<endflags>\w+))?\s+
        (?P<beginchars>\w+)
        (?:/(?P<beginflag>\w+))?
        (?P<replacement>\s\w+)?$"
    )
    .unwrap();
}

impl Conversion {
    /// Create a `Conversion` from a string. Splits on whitespace
    pub fn from_str(value: &str, bidirectional: bool) -> Result<Self, ParseErrorType> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorType::ConversionSplit(split.len()));
        }
        Ok(Self {
            input: split[0].to_owned(),
            output: split[1].to_owned(),
            bidirectional,
        })
    }
}

impl RuleGroup {
    /// Apply a pattern from this rule group to a root string. Returns `None` if
    /// there are no matches.
    ///
    /// Does not pay attention to prf/sfx combinations, that must be done
    /// earlier.
    #[inline]
    pub fn apply_pattern(&self, stem: &str) -> Option<String> {
        self.rules
            .iter()
            .find_map(|rule| rule.apply_pattern(stem, self.kind))
    }

    pub(crate) fn apply_pattern_meta(&self, stem: &str) -> Option<(String, &AffixRule)> {
        self.rules
            .iter()
            .find_map(|rule| rule.apply_pattern(stem, self.kind).map(|s| (s, rule)))
    }
}

impl AffixRule {
    pub fn new(
        kind: RuleType,
        affix: &str,
        strip: Option<&str>,
        condition: Option<&str>,
        morph_info: Vec<MorphInfo>,
    ) -> Result<Self, Error> {
        let cond_re = match condition {
            Some(c) => Self::compile_re_pattern(c, kind)?,
            None => None,
        };

        Ok(Self {
            strip: strip.map(ToOwned::to_owned),
            affix: affix.to_owned(),
            condition: cond_re,
            morph_info,
        })
    }

    /// Compile a regex pattern in the context of an affix. Returns None if
    /// the universal pattern "." is provided
    pub(crate) fn compile_re_pattern(
        condition: &str,
        kind: RuleType,
    ) -> Result<Option<Regex>, regex::Error> {
        if condition == "." {
            return Ok(None);
        }
        let re_pattern = match kind {
            RuleType::Prefix => format!("^{condition}.*$"),
            RuleType::Suffix => format!("^.*{condition}$"),
        };
        Regex::new(re_pattern.as_str()).map(Some)
    }

    /// Helper for testing
    pub(crate) fn set_re_pattern(
        &mut self,
        condition: &str,
        kind: RuleType,
    ) -> Result<(), regex::Error> {
        self.condition = Self::compile_re_pattern(condition, kind)?;
        Ok(())
    }

    /// Check whether a condition is applicable, compile if not
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn check_condition(&self, s: &str) -> bool {
        match &self.condition {
            Some(re) => re.is_match(s),
            None => true,
        }
    }

    // Verify the match condition and apply this rule
    pub fn apply_pattern(&self, s: &str, kind: RuleType) -> Option<String> {
        // No return if condition doesn't match
        if !self.check_condition(s) {
            return None;
        }

        let mut working = s;

        match kind {
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
}

impl MorphInfo {
    pub(crate) fn many_from_str(s: &str) -> Result<Vec<Self>, ParseError> {
        let mut res = Vec::new();
        for morph in s.split_whitespace() {
            res.push(MorphInfo::try_from(morph).map_err(|e| ParseError::new_nospan(e, morph))?);
        }
        Ok(res)
    }
}

/* Trait implementations */

impl TryFrom<&str> for Encoding {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "utf-8" => Ok(Self::Utf8),
            "iso8859-1" => Ok(Self::Iso8859t1),
            "iso8859-10" => Ok(Self::Iso8859t10),
            "iso8859-13" => Ok(Self::Iso8859t13),
            "iso8859-15" => Ok(Self::Iso8859t15),
            "koi8-r" => Ok(Self::Koi8R),
            "koi8-u" => Ok(Self::Koi8U),
            "cp1251" => Ok(Self::Cp1251),
            "iscii-devanagari" => Ok(Self::IsciiDevanagari),
            _ => Err(ParseErrorType::Encoding),
        }
    }
}

impl From<Encoding> for &str {
    #[inline]
    fn from(val: Encoding) -> Self {
        match val {
            Encoding::Utf8 => "UTF-8",
            Encoding::Iso8859t1 => "ISO8859-1",
            Encoding::Iso8859t10 => "ISO8859-10",
            Encoding::Iso8859t13 => "ISO8859-13",
            Encoding::Iso8859t15 => "ISO8859-15",
            Encoding::Koi8R => "KOI8-R",
            Encoding::Koi8U => "KOI8-U",
            Encoding::Cp1251 => "cp1251",
            Encoding::IsciiDevanagari => "ISCII-DEVANAGARI",
        }
    }
}

impl TryFrom<&str> for FlagType {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, ParseErrorType> {
        match value.to_ascii_lowercase().as_str() {
            "ascii" => Ok(Self::Ascii),
            "utf-8" => Ok(Self::Utf8),
            "long" => Ok(Self::Long),
            "num" => Ok(Self::Number),
            _ => Err(ParseErrorType::FlagType),
        }
    }
}

impl From<FlagType> for &str {
    #[inline]
    fn from(val: FlagType) -> Self {
        match val {
            FlagType::Ascii => "ASCII",
            FlagType::Utf8 => "UTF-8",
            FlagType::Long => "long",
            FlagType::Number => "num",
        }
    }
}

impl TryFrom<&str> for Phonetic {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorType::Phonetic(split.len()));
        }
        Ok(Self {
            pattern: split[0].to_owned(),
            replace: split[1].to_owned(),
        })
    }
}

impl TryFrom<&str> for CompoundPattern {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, ParseErrorType> {
        let caps = RE_COMPOUND_PATTERN
            .captures(value)
            .ok_or(ParseErrorType::CompoundPattern)?;
        Ok(Self {
            endchars: caps.name("endchars").unwrap().as_str().to_owned(),
            endflag: caps.name("endflag").map(|m| m.as_str().to_owned()),
            beginchars: caps.name("beginchars").unwrap().as_str().to_owned(),
            beginflag: caps.name("beginflag").map(|m| m.as_str().to_owned()),
            replacement: caps.name("replacement").map(|m| m.as_str().to_owned()),
        })
    }
}

impl TryFrom<&str> for CompoundSyllable {
    type Error = ParseErrorType;

    /// Format: `COMPOUNDSYLLABLE count vowels`
    fn try_from(value: &str) -> Result<Self, ParseErrorType> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorType::CompoundSyllableCount(split.len()));
        }
        let to_parse = split[0];
        let count: u16 = to_parse
            .parse()
            .map_err(ParseErrorType::CompoundSyllableParse)?;
        Ok(Self {
            count,
            vowels: split[1].to_owned(),
        })
    }
}

impl TryFrom<&str> for MorphInfo {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let (tag, val) = value
            .split_once(':')
            .ok_or_else(|| ParseErrorType::MorphInfoDelim(value.to_owned()))?;
        let ret = match tag {
            "st" => Self::Stem(val.to_owned()),
            "ph" => Self::Phonetic(val.to_owned()),
            "al" => Self::Allomorph(val.to_owned()),
            "po" => Self::Part(val.try_into()?),
            "ds" => Self::DerivSfx(val.to_owned()),
            "is" => Self::InflecSfx(val.to_owned()),
            "ts" => Self::TerminalSfx(val.to_owned()),
            "dp" => Self::DerivPfx(val.to_owned()),
            "ip" => Self::InflecPfx(val.to_owned()),
            "tp" => Self::TermPfx(val.to_owned()),
            "sp" => Self::SurfacePfx(val.to_owned()),
            "pa" => Self::CompPart(val.to_owned()),
            _ => return Err(ParseErrorType::MorphInvalidTag(tag.to_owned())),
        };
        Ok(ret)
    }
}

impl TryFrom<&str> for PartOfSpeech {
    type Error = ParseErrorType;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ret = match value.to_lowercase().as_str() {
            "noun" => Self::Noun,
            "verb" => Self::Verb,
            "adjective" => Self::Adjective,
            "determiner" => Self::Determiner,
            "adverb" => Self::Adverb,
            "pronoun" => Self::Pronoun,
            "preposition" => Self::Preposition,
            "conjunction" => Self::Conjunction,
            "interjection" => Self::Interjection,
            _ => return Err(ParseErrorType::PartOfSpeech(value.to_owned())),
        };
        Ok(ret)
    }
}

impl TryFrom<&str> for RuleType {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let ret = match value.to_lowercase().as_str() {
            "pfx" => Self::Prefix,
            "sfx" => Self::Suffix,
            _ => return Err(format!("unrecognized RuleType value '{value}'")),
        };
        Ok(ret)
    }
}

impl Default for Encoding {
    fn default() -> Self {
        Self::Utf8
    }
}

impl Default for FlagType {
    fn default() -> Self {
        Self::Utf8
    }
}

impl PartialEq for AffixRule {
    /// Override default to just check regex string
    fn eq(&self, other: &Self) -> bool {
        self.strip == other.strip
            && self.affix == other.affix
            && self.morph_info == other.morph_info
            && self.condition.as_ref().map(Regex::as_str)
                == other.condition.as_ref().map(Regex::as_str)
    }
}

impl Eq for AffixRule {}

impl Hash for AffixRule {
    /// Hash convert the regex to a string for hashing
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.strip.hash(state);
        self.affix.hash(state);
        self.condition.as_ref().map(Regex::as_str).hash(state);
        self.morph_info.hash(state);
    }
}
