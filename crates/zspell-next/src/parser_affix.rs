//! Module for parsing affix files
//!
//! Contains various munchers for all possible affix keys

mod helpers;
mod types;

use std::fmt::Display;
use std::str::FromStr;

use lazy_static::lazy_static;
use types::AffixNode;

use crate::affix::types::{CompoundPattern, CompoundSyllable, Conversion, Encoding, Phonetic};

/// Characters considered line enders
///
/// We include `#` so comments will get cut off
const LINE_TERMINATORS: [char; 3] = ['\r', '\n', '#'];

/// The result of parsing something
///
/// - `Ok(None)`: nothing found but no errors
/// - `Ok(Some(node, residual))`: matched with result stored to `node`,
///   `residual` contains the rest of the non-matched string
/// - `Err(e)`: error while parsing
type ParseResult<'a> = Result<Option<(AffixNode, &'a str)>, ParseError>;

#[derive(Debug, PartialEq)]
struct ParseError {
    msg: String,
    line_offset: u32,
    col_offset: u32,
}

impl ParseError {
    #[inline]
    fn new_simple<S>(msg: S) -> Self
    where
        S: ToOwned<Owned = String>,
    {
        Self {
            msg: msg.to_owned(),
            line_offset: 0,
            col_offset: 0,
        }
    }

    #[inline]
    fn new<S>(msg: S, line: u32, col: u32) -> Self
    where
        S: ToOwned<Owned = String>,
    {
        Self {
            msg: msg.to_owned(),
            line_offset: line,
            col_offset: col,
        }
    }
}

impl From<String> for ParseError {
    fn from(value: String) -> Self {
        Self::new_simple(value)
    }
}
impl From<&str> for ParseError {
    fn from(value: &str) -> Self {
        Self::new_simple(value.to_owned())
    }
}

/*
    Parser Helpers
*/

/// Split a line by key
///
/// - `None`: key not found
/// - `Some((match, residual))`: `match` is the matched string, `residual` is
///   the leftover
#[inline]
#[allow(clippy::option_if_let_else)]
fn line_splitter<'a>(s: &'a str, key: &str) -> Option<(&'a str, &'a str)> {
    // Skip if we don't start with the key
    if !s.starts_with(key) {
        return None;
    }

    // Parse to newline
    let (work, residual) = match s.find(LINE_TERMINATORS) {
        Some(i) => (&s[key.len()..i], &s[i..]),
        None => (&s[key.len()..], ""),
    };

    Some((work.trim(), residual))
}

/// Parse anything from a given key to the end of a line
///
/// Accepts a string to search, a key to search for, and a function to convert
/// the result type if found
#[inline]
fn line_key_parser<'a, F>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(&str) -> Result<AffixNode, ParseError>,
{
    match line_splitter(s, key) {
        Some((work, residual)) => f(s).map(|n| Some((n, residual))),
        None => Ok(None),
    }
}

/// Parse simple tables
///
/// ```text
/// KEY 4
/// KEY abcd
/// KEY abcd
/// KEY abcd
/// KEY abcd
/// ```
fn table_parser<'a, F>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(Vec<String>) -> Result<AffixNode, ParseError>,
{
    let Some((work, mut residual)) = line_splitter(s, key) else {
        return Ok(None);
    };

    let count: u16 = match work.parse() {
        Ok(v) => v,
        Err(e) => {
            return Err(ParseError::new_simple(format!(
                "error parsing table value: {e}"
            )))
        }
    };

    let mut ret = Vec::new();

    for i in 0..count {
        match line_splitter(residual, key) {
            Some((content, resid)) => {
                residual = resid;
                ret.push(content.to_owned());
            }
            None => {
                return Err(ParseError::new(
                    format!("expected {count} values in table but got {i}"),
                    u32::from(i + 1),
                    0,
                ))
            }
        }
    }

    f(ret).map(|n| Some((n, residual)))
}

/// Parse bool type flag values
///
/// Accepts a string to search, a key to search for, and the node to return if
/// there is no problem
fn bool_parser<'a>(s: &'a str, key: &str, afx: AffixNode) -> ParseResult<'a> {
    line_key_parser(s, key, |s| {
        if s.is_empty() {
            Ok(afx)
        } else {
            Err(ParseError::new_simple(format!(
                "{key} is a boolean flag; nothing else expected on the line but got '{s}'"
            )))
        }
    })
}

/// Parse simple strings
///
/// Accepts a string to search, a key to search for, and a function (enum
/// variant)
fn string_parser<'a, F>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(String) -> AffixNode,
{
    line_key_parser(s, key, |s| Ok(f(s.to_owned())))
}

/// Parse single-character flags
///
/// Accepts a string to search, a key to search for, and a function (enum
/// variant)
fn char_parser<'a, F>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(char) -> AffixNode,
{
    line_key_parser(s, key, |s| {
        let count = s.chars().count();

        if count == 1 {
            Ok(f(s.chars().next().unwrap()))
        } else {
            Err(ParseError::new_simple(format!(
                "expected a single character flag but got {count} chars"
            )))
        }
    })
}

/// Parse integer keys
///
/// Accepts a string to search, a key to search for, and a function (enum
/// variant) that has a parsable type
fn int_parser<'a, F, T, E>(s: &'a str, key: &str, f: F) -> ParseResult<'a>
where
    F: FnOnce(T) -> AffixNode,
    T: FromStr<Err = E>,
    E: Display,
{
    line_key_parser(s, key, |s| {
        s.parse::<T>()
            .map(f)
            .map_err(|e| ParseError::new_simple(format!("failed to parse integer: {e}")))
    })
}

#[inline]
fn convert_u32<T: TryInto<u32> + Display + Copy>(value: T) -> u32 {
    value
        .try_into()
        .unwrap_or_else(|_| panic!("value {value} overflows u32 max of {}", u32::MAX))
}

/*
    General Parsers
*/

/// Consume a comment
fn parse_comment(s: &str) -> ParseResult {
    line_key_parser(s, "#", |s| Ok(AffixNode::Comment))
}
fn parse_encoding(s: &str) -> ParseResult {
    line_key_parser(s, "SET", |s| {
        Encoding::try_from(s)
            .map(AffixNode::Encoding)
            .map_err(ParseError::new_simple)
    })
}
fn parse_flag(s: &str) -> ParseResult {
    line_key_parser(s, "FLAG", |s| {
        Encoding::try_from(s)
            .map(AffixNode::Encoding)
            .map_err(ParseError::new_simple)
    })
}
fn parse_complex_prefixes(s: &str) -> ParseResult {
    bool_parser(s, "COMPLEXPREFIXES", AffixNode::ComplexPrefixes)
}
fn parse_lang(s: &str) -> ParseResult {
    string_parser(s, "LANG", AffixNode::Language)
}
fn parse_ignore_chars(s: &str) -> ParseResult {
    line_key_parser(s, "IGNORE", |s| {
        Ok(AffixNode::IgnoreChars(s.chars().collect()))
    })
}
fn parse_affix_alias(s: &str) -> ParseResult {
    table_parser(s, "AF", |v| {
        for (i, item) in v.iter().enumerate() {
            if item.contains(char::is_whitespace) {
                return Err(ParseError::new(
                    format!("cannot contain whitespace but '{item}' does"),
                    convert_u32(i + 1),
                    0,
                ));
            }
        }
        Ok(AffixNode::AffixAlias(v))
    })
}
fn parse_morph_alias(s: &str) -> ParseResult {
    table_parser(s, "AM", |v| {
        for (i, item) in v.iter().enumerate() {
            if item.contains(char::is_whitespace) {
                return Err(ParseError::new(
                    format!("cannot contain whitespace but '{item}' does"),
                    convert_u32(i + 1),
                    0,
                ));
            }
        }
        Ok(AffixNode::MorphAlias(v))
    })
}

/*
    Suggestion Parsers
*/

fn parse_neighbor_keys(s: &str) -> ParseResult {
    line_key_parser(s, "KEY", |s| {
        Ok(AffixNode::NeighborKeys(
            s.split('|').map(ToOwned::to_owned).collect(),
        ))
    })
}
fn parse_try_characters(s: &str) -> ParseResult {
    string_parser(s, "TRY", AffixNode::TryCharacters)
}
fn parse_nosuggest_flag(s: &str) -> ParseResult {
    char_parser(s, "NOSUGGEST", AffixNode::NoSuggestFlag)
}
fn parse_compound_suggestions_max(s: &str) -> ParseResult {
    int_parser(s, "MAXCPDSUGS", AffixNode::CompoundSuggestionsMax)
}
fn parse_ngram_suggestions_max(s: &str) -> ParseResult {
    int_parser(s, "MAXNGRAMSUGS", AffixNode::NGramSuggestionsMax)
}
fn parse_ngram_diff_max(s: &str) -> ParseResult {
    int_parser(s, "MAXDIFF", AffixNode::NGramDiffMax)
}
fn parse_ngram_limit_to_diff_max(s: &str) -> ParseResult {
    bool_parser(s, "ONLYMAXDIFF", AffixNode::NGramLimitToDiffMax)
}
fn parse_no_split_suggestions(s: &str) -> ParseResult {
    bool_parser(s, "NOSPLITSUGS", AffixNode::NoSplitSuggestions)
}
fn parse_keep_term_dots(s: &str) -> ParseResult {
    bool_parser(s, "SUGSWITHDOTS", AffixNode::KeepTerminationDots)
}
fn parse_replacement(s: &str) -> ParseResult {
    table_parser(s, "REP", |v| {
        let mut res = Vec::new();
        for (i, content) in v.iter().enumerate() {
            res.push(
                Conversion::from_str(content, false)
                    .map_err(|e| ParseError::new(e, convert_u32(i + 1), 0))?,
            );
        }
        Ok(AffixNode::Replacement(res))
    })
}
fn parse_mapping(s: &str) -> ParseResult {
    table_parser(s, "MAP", |v| {
        let mut res = Vec::new();
        for (i, item) in v.iter().enumerate() {
            let mut chars = item.chars();
            res.push(chars.next().zip(chars.next()).ok_or_else(|| {
                ParseError::new(
                    format!("expected two chars but got '{item}'"),
                    convert_u32(i + 1),
                    0,
                )
            })?);
        }
        Ok(AffixNode::Mapping(res))
    })
}
fn parse_phonetic(s: &str) -> ParseResult {
    table_parser(s, "PHONE", |v| {
        let mut res = Vec::new();
        for (i, item) in v.iter().enumerate() {
            match Phonetic::try_from(item.as_str()) {
                Ok(p) => res.push(p),
                Err(e) => return Err(ParseError::new(e, convert_u32(i + 1), 0)),
            }
        }
        Ok(AffixNode::Phonetic(res))
    })
}
fn parse_warn_rare(s: &str) -> ParseResult {
    char_parser(s, "WARN", AffixNode::WarnRareFlag)
}

/*
    Compounding Parsers
*/

fn parse_forbidden_warn(s: &str) -> ParseResult {
    bool_parser(s, "FORBIDWARN", AffixNode::ForbidWarnWords)
}
fn parse_break_separator(s: &str) -> ParseResult {
    table_parser(s, "BREAK", |v| {
        for (i, item) in v.iter().enumerate() {
            if item.contains(char::is_whitespace) {
                return Err(ParseError::new(
                    format!("cannot contain whitespace but '{item}' does"),
                    convert_u32(i + 1),
                    0,
                ));
            }
        }
        Ok(AffixNode::BreakSeparator(v))
    })
}
fn parse_compound_rule(s: &str) -> ParseResult {
    table_parser(s, "COMPOUNDRULE", |v| {
        for (i, item) in v.iter().enumerate() {
            if item.contains(char::is_whitespace) {
                return Err(ParseError::new(
                    format!("cannot contain whitespace but '{item}' does"),
                    convert_u32(i + 1),
                    0,
                ));
            }
        }
        Ok(AffixNode::BreakSeparator(v))
    })
}
fn parse_compound_min_length(s: &str) -> ParseResult {
    int_parser(s, "COMPOUNDMIN", AffixNode::CompoundMinLength)
}
fn parse_compound_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDFLAG", AffixNode::CompoundFlag)
}
fn parse_compound_begin_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDBEGIN", AffixNode::CompoundBeginFlag)
}
fn parse_compound_end_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDLAST", AffixNode::CompoundEndFlag)
}
fn parse_compound_middle_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDMIDDLE", AffixNode::CompoundMiddleFlag)
}
fn parse_compound_only_flag(s: &str) -> ParseResult {
    char_parser(s, "ONLYINCOMPOUND", AffixNode::CompoundOnlyFlag)
}
fn parse_compound_permit_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDPERMITFLAG", AffixNode::CompoundPermitFlag)
}
fn parse_compound_forbid_flag(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDFORBIDFLAG", AffixNode::CompoundForbidFlag)
}
fn parse_compound_more_suffixes(s: &str) -> ParseResult {
    bool_parser(s, "COMPOUNDMORESUFFIXES", AffixNode::CompoundMoreSuffixes)
}
fn parse_compound_root(s: &str) -> ParseResult {
    char_parser(s, "COMPOUNDROOT", AffixNode::CompoundRoot)
}
fn parse_compound_word_max(s: &str) -> ParseResult {
    int_parser(s, "COMPOUNDWORDMAX", AffixNode::CompoundWordMax)
}
fn parse_compound_forbid_duplication(s: &str) -> ParseResult {
    bool_parser(s, "CHECKCOMPOUNDDUP", AffixNode::CompoundForbidDuplication)
}
fn parse_compound_forbid_repeat(s: &str) -> ParseResult {
    bool_parser(s, "CHECKCOMPOUNDREP", AffixNode::CompoundForbidRepeat)
}
fn parse_compound_check_case(s: &str) -> ParseResult {
    bool_parser(s, "CHECKCOMPOUNDCASE", AffixNode::CompoundCheckCase)
}
fn parse_compound_check_triple(s: &str) -> ParseResult {
    bool_parser(s, "CHECKCOMPOUNDTRIPLE", AffixNode::CompoundCheckTriple)
}
fn parse_compound_simplify_triple(s: &str) -> ParseResult {
    bool_parser(s, "SIMPLIFIEDTRIPLE", AffixNode::CompoundSimplifyTriple)
}
fn parse_compound_forbid_patterns(s: &str) -> ParseResult {
    table_parser(s, "CHECKCOMPOUNDPATTERN", |v| {
        let mut res = Vec::new();
        for (i, item) in v.iter().enumerate() {
            res.push(
                CompoundPattern::try_from(item.as_str())
                    .map_err(|e| ParseError::new(e, convert_u32(i + 1), 0))?,
            )
        }
        Ok(AffixNode::CompoundForbidPatterns(res))
    })
}
fn parse_compound_force_upper(s: &str) -> ParseResult {
    char_parser(s, "FORCEUCASE", AffixNode::CompoundForceUpper)
}
fn parse_compound_syllable(s: &str) -> ParseResult {
    line_key_parser(s, "COMPOUNDSYLLABLE", |s| {
        Ok(AffixNode::CompoundSyllable(
            CompoundSyllable::try_from(s).map_err(ParseError::new_simple)?,
        ))
    })
}
fn parse_syllable_num(s: &str) -> ParseResult {
    string_parser(s, "SYLLABLENUM", AffixNode::SyllableNum)
}

/*
    Affix Parsers
*/

fn parse_prefix(s: &str) -> ParseResult {
    todo!()
}
fn parse_suffix(s: &str) -> ParseResult {
    todo!()
}

/*
    Other Parsers
*/

fn parse_circumfix_flag(s: &str) -> ParseResult {
    char_parser(s, "CIRCUMFIX", AffixNode::AffixCircumfixFlag)
}
fn parse_forbidden_word_flag(s: &str) -> ParseResult {
    char_parser(s, "FORBIDDENWORD", AffixNode::ForbiddenWordFlag)
}
fn parse_afx_full_strip(s: &str) -> ParseResult {
    bool_parser(s, "FULLSTRIP", AffixNode::AffixFullStrip)
}
fn parse_afx_keep_case_flag(s: &str) -> ParseResult {
    char_parser(s, "KEEPCASE", AffixNode::AffixKeepCaseFlag)
}
fn parse_afx_input_conversion(s: &str) -> ParseResult {
    table_parser(s, "ICONV", |v| {
        let mut res = Vec::new();
        for (i, content) in v.iter().enumerate() {
            res.push(
                Conversion::from_str(content, false)
                    .map_err(|e| ParseError::new(e, (i + 1).try_into().unwrap(), 0))?,
            );
        }
        Ok(AffixNode::AffixInputConversion(res))
    })
}
fn parse_afx_output_conversion(s: &str) -> ParseResult {
    table_parser(s, "OCONV", |v| {
        let mut res = Vec::new();
        for (i, content) in v.iter().enumerate() {
            res.push(
                Conversion::from_str(content, false)
                    .map_err(|e| ParseError::new(e, (i + 1).try_into().unwrap(), 0))?,
            );
        }
        Ok(AffixNode::AffixOutputConversion(res))
    })
}
fn parse_afx_lemma_present_flag(s: &str) -> ParseResult {
    char_parser(s, "LEMMA_PRESENT", AffixNode::AffixLemmaPresentFlag)
}
fn parse_afx_needed_flag(s: &str) -> ParseResult {
    char_parser(s, "NEEDAFFIX", AffixNode::AffixNeededFlag)
}
fn parse_afx_pseudoroot_flag(s: &str) -> ParseResult {
    char_parser(s, "PSEUDOROOT", AffixNode::AffixPseudoRootFlag)
}
fn parse_afx_substandard_flag(s: &str) -> ParseResult {
    char_parser(s, "SUBSTANDARD", AffixNode::AffixSubstandardFlag)
}
fn parse_afx_word_chars(s: &str) -> ParseResult {
    string_parser(s, "WORDCHARS", AffixNode::AffixWordChars)
}
fn parse_afx_check_sharps(s: &str) -> ParseResult {
    bool_parser(s, "CHECKSHARPS", AffixNode::AffixCheckSharps)
}
fn parse_name(s: &str) -> ParseResult {
    string_parser(s, "NAME", AffixNode::Name)
}
fn parse_home(s: &str) -> ParseResult {
    string_parser(s, "HOME", AffixNode::HomePage)
}
fn parse_version(s: &str) -> ParseResult {
    string_parser(s, "VERSION", AffixNode::Version)
}

const ALL_PARSERS: [for<'a> fn(&'a str) -> ParseResult; 61] = [
    parse_comment,
    parse_encoding,
    parse_flag,
    parse_complex_prefixes,
    parse_lang,
    parse_ignore_chars,
    parse_affix_alias,
    parse_morph_alias,
    parse_neighbor_keys,
    parse_try_characters,
    parse_nosuggest_flag,
    parse_compound_suggestions_max,
    parse_ngram_suggestions_max,
    parse_ngram_diff_max,
    parse_ngram_limit_to_diff_max,
    parse_no_split_suggestions,
    parse_keep_term_dots,
    parse_replacement,
    parse_mapping,
    parse_phonetic,
    parse_warn_rare,
    parse_forbidden_warn,
    parse_break_separator,
    parse_compound_rule,
    parse_compound_min_length,
    parse_compound_flag,
    parse_compound_begin_flag,
    parse_compound_end_flag,
    parse_compound_middle_flag,
    parse_compound_only_flag,
    parse_compound_permit_flag,
    parse_compound_forbid_flag,
    parse_compound_more_suffixes,
    parse_compound_root,
    parse_compound_word_max,
    parse_compound_forbid_duplication,
    parse_compound_forbid_repeat,
    parse_compound_check_case,
    parse_compound_check_triple,
    parse_compound_simplify_triple,
    parse_compound_forbid_patterns,
    parse_compound_force_upper,
    parse_compound_syllable,
    parse_syllable_num,
    parse_prefix,
    parse_suffix,
    parse_circumfix_flag,
    parse_forbidden_word_flag,
    parse_afx_full_strip,
    parse_afx_keep_case_flag,
    parse_afx_input_conversion,
    parse_afx_output_conversion,
    parse_afx_lemma_present_flag,
    parse_afx_needed_flag,
    parse_afx_pseudoroot_flag,
    parse_afx_substandard_flag,
    parse_afx_word_chars,
    parse_afx_check_sharps,
    parse_name,
    parse_home,
    parse_version,
];

#[cfg(test)]
mod tests;
