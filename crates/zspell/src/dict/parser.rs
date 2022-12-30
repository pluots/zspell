//! Functions for parsing a dic file

use lazy_static::lazy_static;
use regex::Regex;

use crate::affix::FlagType;
use crate::error::{ParseError, ParseErrorKind};
use crate::helpers::convertu32;
use crate::morph::MorphInfo;

lazy_static! {
    static ref RE_DICT_LINE: Regex = Regex::new(
        r"(?x)
        ^
        (?P<stem>\S+?)
        (?:/
            (?P<flags>\S+)
        )?
        (?:\s+
            (?P<morph>[\s\w:]+?)
        )?
        (?:\s+\#.*)?$
    "
    )
    .unwrap();
    static ref RE_PERSONAL_LINE: Regex = Regex::new(
        r"(?x)
        ^
        (?P<forbid>\*)?
        (?P<stem>\S+?)
        (?:/
            (?P<friend>\w+)
        )?
        (?:\s+
            (?P<morph>[\s\w:]+?)
        )?
        (?:\s+\#.*)?$
    "
    )
    .unwrap();
}

/// Represent a single line in a dictionary file
///
/// Format is as follows:
///
/// ```text
/// word[/flags...] [morphinfo ...]
/// band/ESGD po:noun
/// laser/M
/// ```
/// Flags and morph info are optional
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DictEntry {
    pub(super) stem: String,
    pub(super) flags: Vec<u32>,
    pub(super) morph: Vec<MorphInfo>,
}

impl DictEntry {
    /// Create a new `DictEntry`
    #[cfg(test)]
    pub(crate) fn new(stem: String, flags: &[u32], morph: Vec<MorphInfo>) -> Self {
        Self {
            stem,
            flags: flags.to_owned(),
            morph,
        }
    }

    /// Create a `DictEntry` from a single line in a `.dic` file
    pub(crate) fn parse_str(
        value: &str,
        flag_type: FlagType,
        line_num: u32,
    ) -> Result<Self, ParseError> {
        let Some(caps) = RE_DICT_LINE.captures(value) else {
            return Err(ParseError::new_nocol(
                ParseErrorKind::DictEntry,
                value,
                line_num,
            ));
        };

        let stem = caps.name("stem").unwrap().as_str().to_owned();
        let flags: Vec<u32> = match caps.name("flags") {
            Some(flagstr) => flag_type
                .parse_str(flagstr.as_str())
                .map_err(|e| ParseError::new_nospan(e, flagstr.as_str()))?,
            None => Vec::new(),
        };

        let morph = if let Some(morphstr) = caps.name("morph") {
            MorphInfo::many_from_str(morphstr.as_str())?
        } else {
            Vec::new()
        };

        Ok(Self { stem, flags, morph })
    }

    pub fn stem(&self) -> &str {
        &self.stem
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
    pub stem: String,
    pub friend: Option<String>,
    pub morph: Vec<MorphInfo>,
    pub forbid: bool,
}

impl PersonalEntry {
    pub(crate) fn new(
        stem: &str,
        friend: Option<&str>,
        morph: Vec<MorphInfo>,
        forbid: bool,
    ) -> Self {
        Self {
            stem: stem.to_owned(),
            friend: friend.map(ToOwned::to_owned),
            morph,
            forbid,
        }
    }

    pub(crate) fn parse_str(value: &str, line_num: u32) -> Result<Self, ParseError> {
        let Some(caps) = RE_PERSONAL_LINE.captures(value) else {
            return Err(ParseError::new_nocol(
                ParseErrorKind::Personal,
                value,
                line_num,
            ));
        };

        let forbid = caps.name("forbid").is_some();
        let stem = caps.name("stem").unwrap().as_str().to_owned();
        let friend = caps.name("friend").map(|m| m.as_str().to_owned());
        let morph = if let Some(morphstr) = caps.name("morph") {
            MorphInfo::many_from_str(morphstr.as_str())?
        } else {
            Vec::new()
        };

        Ok(Self {
            stem,
            friend,
            morph,
            forbid,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParsedPersonalMeta {
    friend: Option<String>,
    morph: Vec<MorphInfo>,
}

impl ParsedPersonalMeta {
    pub(crate) fn new<S: AsRef<str>>(friend: Option<S>, morph: Vec<MorphInfo>) -> Self {
        Self {
            friend: friend.map(|s| s.as_ref().to_owned()),
            morph,
        }
    }
}

/// Parse a complete dictionary file (usually `.dic`)
#[allow(clippy::single_match_else, clippy::option_if_let_else)]
pub fn parse_dict(s: &str, flag_type: FlagType) -> Result<Vec<DictEntry>, ParseError> {
    // Ignore empty lines and
    let mut lines_iter = s
        .lines()
        .map(str::trim)
        .filter(|line| !(line.is_empty() || line.starts_with('#')));
    let lines_backup = lines_iter.clone();

    let Some(first) = lines_iter.next() else {
        return Ok(Vec::new())
    };

    // Try to parse the first line as an integer; if not, ignore it
    let (mut ret, start) = match first.parse::<usize>() {
        Ok(cap) => (Vec::with_capacity(cap), 2),
        Err(_) => {
            lines_iter = lines_backup;
            (Vec::new(), 1)
        }
    };

    for (i, line) in lines_iter.enumerate() {
        ret.push(DictEntry::parse_str(
            line,
            flag_type,
            convertu32(i + start),
        )?);
    }
    Ok(ret)
}

/// Parse a personal dictionary file
pub fn parse_personal_dict(s: &str) -> Result<Vec<PersonalEntry>, ParseError> {
    let mut ret = Vec::new();

    for (i, line) in s.lines().map(str::trim).enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        ret.push(PersonalEntry::parse_str(line, convertu32(i))?);
    }
    Ok(ret)
}
