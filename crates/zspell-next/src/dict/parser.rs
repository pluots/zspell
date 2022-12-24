//! Functions for parsing a dic file

use lazy_static::lazy_static;
use regex::Regex;

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

struct DictEntry {
    stem: String,
    flags: Vec<String>,
    morph: Option<MorphInfo>,
}

impl DictEntry {
    fn parse_str(value: &str, line_num: u32) -> Result<Self, ParseError> {
        let Some(caps) = RE_DICT_LINE.captures(value) else {
            return Err(ParseError::new(
                ParseErrorType::DictEntry(value.to_owned()),
                line_num,
                0,
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
        // let morph = caps.name("morph").map(|m| MorphInfo::try_from(m.as_str())).transpose();
        let morph = None;

        Ok(Self { stem, flags, morph })
    }
}

fn parse_dict(s: &str) -> Result<Vec<DictEntry>, ParseError> {
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

    for (i, line) in s.lines().map(|l| l.trim()).enumerate() {
        if line.starts_with('#') {
            continue;
        }

        ret.push(DictEntry::parse_str(line, convertu32(i + start as usize))?);
    }
    Ok(ret)
}
