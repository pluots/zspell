//! Parse a dict file

use std::sync::Arc;

use crate::affix::FlagType;
use crate::error::ParseError;
use crate::helpers::convertu32;
use crate::morph::MorphInfo;

/// Represent a single line in a dictionary file
///
/// Format is as follows:
///
/// ```text
/// word[/flags...] [morphinfo ...]
/// band/ESGD po:noun
/// laser/M
/// fruit
/// ```
/// Flags and morph info are optional
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DictEntry {
    pub(super) stem: Arc<str>,
    pub(super) flags: Vec<u32>,
    pub(super) morph: Vec<Arc<MorphInfo>>,
}

impl DictEntry {
    /// Test config: create a new `DictEntry`
    #[cfg(test)]
    pub(crate) fn new(stem: &str, flags: &[u32], morph: &[MorphInfo]) -> Self {
        Self {
            stem: stem.into(),
            flags: flags.to_owned(),
            morph: morph.iter().map(|v| Arc::new(v.clone())).collect(),
        }
    }

    /// Create a `DictEntry` from a single line in a `.dic` file. Does not strip comments.
    fn parse_single(value: &str, flag_type: FlagType, line_num: u32) -> Result<Self, ParseError> {
        let (stem, flagstr, morphstr) = separate_into_parts(value);

        let flags: Vec<u32> = match flagstr {
            Some(s) => flag_type
                .parse_str(s.trim())
                .map_err(|e| ParseError::new_nocol(e, s, line_num))?,
            None => Vec::new(),
        };
        let morph = MorphInfo::many_from_str(morphstr.trim())
            .map(Arc::new)
            .collect();
        let ret = Self {
            stem: stem.trim().into(),
            flags,
            morph,
        };
        Ok(ret)
    }

    /// Parse a complete dictionary file (usually `.dic`)
    ///
    /// # Errors
    ///
    /// Returns an error if any entry is incorrect.
    #[inline]
    #[allow(clippy::option_if_let_else)]
    pub fn parse_all(input: &str, flag_type: FlagType) -> Result<Vec<DictEntry>, ParseError> {
        // Ignore empty lines and
        let mut lines_iter = extract_content(input);
        let lines_backup = lines_iter.clone();

        let Some(first) = lines_iter.next() else {
            return Ok(Vec::new());
        };

        // Try to parse the first line as an integer; if not, ignore it
        let (mut ret, start) = if let Ok(cap) = first.parse::<usize>() {
            (Vec::with_capacity(cap), 2)
        } else {
            lines_iter = lines_backup;
            (Vec::new(), 1)
        };

        for (i, line) in lines_iter.enumerate() {
            ret.push(
                DictEntry::parse_single(line, flag_type, convertu32(i + start))
                    .map_err(|e| e.add_offset_ret(i + start, 0))?,
            );
        }
        Ok(ret)
    }
}

/// Represent an entry from a personal dictionary
///
/// Format is as follows:
///
/// ```text
/// [*]word[/friend] [morphinfo ...]
/// enum/apple po:noun
/// someword
/// *ignoreword
/// ```
///
/// The hunspell spec doesn't say anything about morph info, but why not allow
/// it
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PersonalEntry {
    pub stem: Arc<str>,
    pub friend: Option<Box<str>>,
    pub morph: Vec<MorphInfo>,
    pub forbid: bool,
}

impl PersonalEntry {
    #[cfg(test)]
    pub(crate) fn new(
        stem: &str,
        friend: Option<&str>,
        morph: Vec<MorphInfo>,
        forbid: bool,
    ) -> Self {
        Self {
            stem: stem.into(),
            friend: friend.map(Into::into),
            morph,
            forbid,
        }
    }

    pub fn parse_single(value: &str) -> Self {
        let (stem, friend, morphstr) = separate_into_parts(value);
        let forbid = stem.starts_with('*');
        let stem = stem.strip_prefix('*').unwrap_or(stem);
        let morph = MorphInfo::many_from_str(morphstr).collect();

        Self {
            stem: stem.trim().into(),
            friend: friend.map(|f| f.trim().into()),
            morph,
            forbid,
        }
    }
    /// Parse a personal dictionary file
    pub fn parse_all(s: &str) -> Vec<PersonalEntry> {
        extract_content(s).map(Self::parse_single).collect()
    }
}

/// Separate `(stem, flagstr, morphstr)` into parts
fn separate_into_parts(value: &str) -> (&str, Option<&str>, &str) {
    let stem: &str;
    let flagstr: Option<&str>;
    let morphstr: &str;

    let value = value.split_once('#').unwrap_or((value, "")).0;

    // Split out the sections
    if let Some((word, rest)) = value.split_once('/') {
        // Easy case, we have an affix and can split on `/`. Then just split the first
        // whitespace to separate morph from the flags.
        stem = word;
        let (tmpflag, tmpmorph) = rest
            .split_once(|ch: char| ch.is_ascii_whitespace())
            .unwrap_or((rest, ""));
        flagstr = Some(tmpflag);
        morphstr = tmpmorph;
    } else {
        // Trickier case; we look for a colon, find whitespace to the left, and assume
        // everything to the left of that is the word
        flagstr = None;
        (stem, morphstr) = value.find(':').map_or((value, ""), |idx| {
            value[..idx]
                .rfind(|ch: char| ch.is_ascii_whitespace())
                .map_or((value, ""), |ws_idx| (&value[..ws_idx], &value[ws_idx..]))
        });
    };
    (stem, flagstr, morphstr)
}

/// Extract nonempty lines that do not contain a comment
fn extract_content(input: &str) -> impl Iterator<Item = &str> + Clone {
    input
        .lines()
        // Dictionary files sometimes use tabs for comments, need to check before trim
        .filter(|line| !line.starts_with('\t'))
        // Trim hash comments
        .map(|line| line.split_once('#').unwrap_or((line, "")).0)
        .map(str::trim)
        .filter(|line| !line.trim().is_empty())
}

#[cfg(test)]
#[path = "tests_parse.rs"]
mod tests;
