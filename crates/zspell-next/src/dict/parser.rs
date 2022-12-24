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
}

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

        let mut morph = Vec::new();
        if let Some(split) = caps.name("morph").map(|m| m.as_str().split_whitespace()) {
            for m in split {
                morph.push(MorphInfo::try_from(m).map_err(|e| ParseError::new_nospan(e, m))?);
            }
        };

        Ok(Self { stem, flags, morph })
    }
}

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
