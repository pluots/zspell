//! Functions for parsing a dic file

use lazy_static::lazy_static;
use regex::Regex;

use super::Dictionary;
use crate::affix::types::MorphInfo;
use crate::error::{Error, ParseError, ParseErrorType};
use crate::helpers::convertu32;

lazy_static! {
    static ref RE_DICT_LINE: Regex = Regex::new(
        r"(?x)
        ^
        (?P<stem>\S+?)
        (?:/
            (?P<flags>\w+)
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
pub(crate) struct DictEntry {
    pub stem: String,
    pub flags: Vec<String>,
    pub morph: Vec<MorphInfo>,
}

impl DictEntry {
    pub(crate) fn new(stem: String, flags: Vec<String>, morph: Vec<MorphInfo>) -> Self {
        Self { stem, flags, morph }
    }

    /// Create a `DictEntry` from a line in a `.dic` file
    pub(crate) fn parse_str(value: &str, line_num: u32) -> Result<Self, ParseError> {
        let Some(caps) = RE_DICT_LINE.captures(value) else {
            return Err(ParseError::new_nocol(
                ParseErrorType::DictEntry,
                value,
                line_num,
            ));
        };

        let stem = caps.name("stem").unwrap().as_str().to_owned();
        let flags = caps
            .name("flags")
            .map(|flags| {
                flags
                    .as_str()
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
            })
            .unwrap_or_default();

        let morph = if let Some(morphstr) = caps.name("morph") {
            MorphInfo::many_from_str(morphstr.as_str())?
        } else {
            Vec::new()
        };

        Ok(Self { stem, flags, morph })
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
pub(crate) struct PersonalEntry {
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
                ParseErrorType::Personal,
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
pub struct PersonalMeta {
    pub friend: Option<String>,
    pub morph: Vec<MorphInfo>,
}

impl PersonalMeta {
    pub(crate) fn new<S: AsRef<str>>(friend: Option<S>, morph: Vec<MorphInfo>) -> Self {
        Self {
            friend: friend.map(|s| s.as_ref().to_owned()),
            morph,
        }
    }
}

/// Parse a dictionary file (usually `.dic`)
#[allow(clippy::single_match_else, clippy::option_if_let_else)]
pub(crate) fn parse_dict(s: &str) -> Result<Vec<DictEntry>, ParseError> {
    let mut lines = s.lines();
    let Some(first) = lines.next() else {
        return Ok(Vec::new())
    };

    // Try to parse the first line as an integer; if not, ignore it
    let (mut ret, start) = match first.parse::<usize>() {
        Ok(cap) => (Vec::with_capacity(cap), 2),
        Err(_) => {
            lines = s.lines();
            (Vec::new(), 1)
        }
    };

    for (i, line) in s.lines().map(str::trim).enumerate() {
        if line.starts_with('#') {
            continue;
        }

        ret.push(DictEntry::parse_str(line, convertu32(i + start))?);
    }
    Ok(ret)
}

/// Parse a personal dictionary file
pub(crate) fn parse_personal_dict(s: &str) -> Result<Vec<PersonalEntry>, ParseError> {
    let mut ret = Vec::new();

    for (i, line) in s.lines().map(str::trim).enumerate() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }

        ret.push(PersonalEntry::parse_str(line, convertu32(i))?);
    }
    Ok(ret)
}
