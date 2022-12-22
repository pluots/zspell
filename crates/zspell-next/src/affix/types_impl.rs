//! Extension of the `types` module containing the messy impl blocks

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use super::types::{CompoundPattern, CompoundSyllable, Conversion, Encoding, Flag, Phonetic};

lazy_static! {
    static ref RE_COMPOUND_PATTERN: Regex = Regex::new(r"^(?P<endchars>\w+)(?:/(?P<endflags>\w+))?\s+(?P<beginchars>\w+)(?:/(?P<beginflag>\w+))?(?P<replacement>\s\w+)?$").unwrap();
}

impl TryFrom<&str> for Encoding {
    type Error = String;

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
            _ => Err(format!("unrecognized encoding {value}")),
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

impl TryFrom<&str> for Flag {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_ascii_lowercase().as_str() {
            "ascii" => Ok(Self::Ascii),
            "utf-8" => Ok(Self::Utf8),
            "long" => Ok(Self::Long),
            "num" => Ok(Self::Number),
            _ => Err(format!("unrecognized flag {value}")),
        }
    }
}

impl From<Flag> for &str {
    #[inline]
    fn from(val: Flag) -> Self {
        match val {
            Flag::Ascii => "ASCII",
            Flag::Utf8 => "UTF-8",
            Flag::Long => "long",
            Flag::Number => "num",
        }
    }
}

impl TryFrom<&str> for Phonetic {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(format!("expected 2 items but got {}", split.len()));
        }
        Ok(Self {
            pattern: split[0].to_owned(),
            replace: split[1].to_owned(),
        })
    }
}

impl Conversion {
    /// Create a `Conversion` from a string. Splits on whitespace
    pub fn from_str(value: &str, bidirectional: bool) -> Result<Self, String> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(format!("expected 2 items but got {}", split.len()));
        }
        Ok(Self {
            input: split[0].to_owned(),
            output: split[1].to_owned(),
            bidirectional,
        })
    }
}

impl TryFrom<&str> for CompoundPattern {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let caps = RE_COMPOUND_PATTERN
            .captures(value)
            .ok_or(format!("cannot parse compound pattern at '{value}'"))?;
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
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(format!("expected 2 items but got {}", split.len()));
        }
        let to_parse = split[0];
        let count: u16 = to_parse
            .parse()
            .map_err(|e| format!("unable to parse integer at {to_parse}: {e}"))?;
        Ok(Self {
            count,
            vowels: split[1].to_owned(),
        })
    }
}
