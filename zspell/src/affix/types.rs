//! Type representations for affix file contents

use std::fmt::{self, Display};
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::Regex;

use crate::dict::Flag;
use crate::error::ParseErrorKind;
use crate::morph::MorphStr;

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

/// A possible encoding type
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Encoding {
    /// UTF-8 encoding
    Utf8,
    /// ISO8859-1 encoding
    Iso8859t1,
    /// ISO8859-10 encoding
    Iso8859t10,
    /// ISO8859-13 encoding
    Iso8859t13,
    /// ISO8859-15 encoding
    Iso8859t15,
    /// KOI8-R encoding
    Koi8R,
    /// KOI8-U encoding
    Koi8U,
    /// cp1251 encoding
    Cp1251,
    /// ISCII-DEVANAGARI encoding
    IsciiDevanagari,
}

/// A representation of the flag type (the part after `/` in the `.dic` file)
///
/// We represent all flag types as a u32 and provide methods of conversion
#[non_exhaustive]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlagType {
    /// Single-character ASCII flags (default, single byte)
    Ascii,
    /// Single-character UTF8 flags (up to 4 bytes)
    Utf8,
    /// Double extended ASCII flags, i.e., two ASCII characters (2 bytes)
    Long,
    /// Decimal flag type (we use u32)
    Number,
}

impl FlagType {
    /// Convert a string flag to its u32 representation
    pub(crate) fn str_to_flag(self, flag: &str) -> Result<Flag, ParseErrorKind> {
        match self {
            // Single ascii char
            FlagType::Ascii => Self::parse_as_ascii(flag),
            // Single unicode character
            FlagType::Utf8 => Self::parse_as_utf8(flag),
            // Two asii chars
            FlagType::Long => Self::parse_as_long(flag),
            FlagType::Number => Self::parse_as_number(flag),
        }
    }

    /// Parse a string to multiple flags as they are defined in the dictionary
    /// file
    ///
    /// ASCII and UTF-8 flags just split by characters. Long splits every two
    /// characters, numbers split by commas
    pub(crate) fn parse_str(self, s: &str) -> Result<Vec<Flag>, ParseErrorKind> {
        match self {
            FlagType::Ascii => s.chars().map(Self::parse_char_ascii).collect(),
            FlagType::Utf8 => Ok(s.chars().map(Self::parse_char_utf8).collect()),
            FlagType::Number => s.split(',').map(|flag| self.str_to_flag(flag)).collect(),
            FlagType::Long => {
                let mut ret = Vec::with_capacity(s.len() / 2);
                let mut iter = s.chars();
                for ch in s.chars() {
                    let ch_next = iter.next().ok_or(ParseErrorKind::FlagParse(self))?;
                    ret.push(Self::parse_chars_long([ch, ch_next])?);
                }
                Ok(ret)
            }
        }
    }

    fn parse_as_ascii(flag: &str) -> Result<Flag, ParseErrorKind> {
        if flag.len() == 1 {
            Ok(Flag(flag.bytes().next().unwrap().into()))
        } else {
            Err(ParseErrorKind::FlagParse(Self::Ascii))
        }
    }

    fn parse_as_utf8(flag: &str) -> Result<Flag, ParseErrorKind> {
        let mut chars = flag.chars();
        let err = Err(ParseErrorKind::FlagParse(Self::Utf8));

        let Some(ch) = chars.next() else {
            return err;
        };

        if chars.next().is_some() {
            return err;
        }

        Ok(Flag(ch.into()))
    }

    /// Parse two ascii characters
    fn parse_as_long(flag: &str) -> Result<Flag, ParseErrorKind> {
        if flag.len() != 2 || flag.chars().any(|c| !c.is_ascii()) {
            Err(ParseErrorKind::FlagParse(Self::Long))
        } else {
            let v = u16::from_ne_bytes(flag[0..=1].as_bytes().try_into().unwrap());
            Ok(Flag(v.into()))
        }
    }

    /// Parse as a number
    fn parse_as_number(flag: &str) -> Result<Flag, ParseErrorKind> {
        flag.parse()
            .map_err(|_| ParseErrorKind::FlagParse(Self::Number))
            .map(Flag)
    }

    fn parse_char_ascii(c: char) -> Result<Flag, ParseErrorKind> {
        if c.is_ascii() {
            Ok(Flag(c.into()))
        } else {
            Err(ParseErrorKind::FlagParse(Self::Ascii))
        }
    }

    fn parse_char_utf8(c: char) -> Flag {
        Flag(c.into())
    }

    fn parse_chars_long(chars: [char; 2]) -> Result<Flag, ParseErrorKind> {
        if chars.iter().any(|ch| !ch.is_ascii()) {
            Err(ParseErrorKind::FlagParse(Self::Long))
        } else {
            let arr = [chars[0].try_into().unwrap(), chars[1].try_into().unwrap()];
            Ok(Flag(u16::from_ne_bytes(arr).into()))
        }
    }

    /// Given a specified flag type (self), turn the value back into a string
    #[inline]
    pub fn flag_to_str(self, flag: Flag) -> String {
        match self {
            // Should be OK to unwrap because we created these flags from valid characters
            FlagType::Ascii | FlagType::Utf8 => char::from_u32(flag.0).unwrap().to_string(),
            FlagType::Number => flag.0.to_string(),
            FlagType::Long => {
                let bytes = (u16::try_from(flag.0).unwrap()).to_ne_bytes();
                bytes.iter().map(|b| *b as char).collect::<String>()
            }
        }
    }
}

/// A simple input-to-output conversion mapping.
///
/// This is usually represented in an affix file via `REP`, `ICONV`, and
/// `OCONV`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conversion {
    input: String,
    output: String,
    bidirectional: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CompoundSyllable {
    count: u16,
    vowels: String,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum RuleType {
    Prefix,
    Suffix,
}

/// Representation of a part of speech
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PartOfSpeech {
    Noun,
    Verb,
    Adjective,
    Determiner,
    Adverb,
    Pronoun,
    Preposition,
    Conjunction,
    Interjection,
    /// Other parts of speech that don't have formal specifiers
    Other(MorphStr),
}

/// Representation of the `PHONE` rule
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Phonetic {
    pattern: String,
    replace: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompoundPattern {
    endchars: String,
    endflag: Option<String>,
    beginchars: String,
    beginflag: Option<String>,
    replacement: Option<String>,
}

/* Method implementations */

impl Phonetic {
    #[allow(unused)]
    pub(crate) fn new(pattern: &str, replace: &str) -> Self {
        Self {
            pattern: pattern.to_owned(),
            replace: replace.to_owned(),
        }
    }
}

impl Conversion {
    #[allow(unused)]
    pub(crate) fn new(input: &str, output: &str, bidirectional: bool) -> Self {
        Self {
            input: input.to_owned(),
            output: output.to_owned(),
            bidirectional,
        }
    }
    /// Create a `Conversion` from a string. Splits on whitespace
    pub fn from_str(value: &str, bidirectional: bool) -> Result<Self, ParseErrorKind> {
        let split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorKind::ConversionSplit(split.len()));
        }
        Ok(Self {
            input: split[0].to_owned(),
            output: split[1].to_owned(),
            bidirectional,
        })
    }
}

/* Trait implementations */

impl TryFrom<&str> for Encoding {
    type Error = ParseErrorKind;

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
            _ => Err(ParseErrorKind::Encoding),
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

impl FromStr for FlagType {
    type Err = ParseErrorKind;

    #[inline]
    fn from_str(value: &str) -> Result<Self, ParseErrorKind> {
        match value.to_ascii_lowercase().as_str() {
            "ascii" => Ok(Self::Ascii),
            "utf-8" => Ok(Self::Utf8),
            "long" => Ok(Self::Long),
            "num" => Ok(Self::Number),
            _ => Err(ParseErrorKind::FlagType),
        }
    }
}

impl Display for FlagType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s: &str = self.into();
        write!(f, "{s}")?;
        Ok(())
    }
}

impl From<&FlagType> for &str {
    #[inline]
    fn from(val: &FlagType) -> Self {
        match val {
            FlagType::Ascii => "ASCII",
            FlagType::Utf8 => "UTF-8",
            FlagType::Long => "long",
            FlagType::Number => "num",
        }
    }
}

impl TryFrom<&str> for Phonetic {
    type Error = ParseErrorKind;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorKind::Phonetic(split.len()));
        }
        Ok(Self {
            pattern: split[0].to_owned(),
            replace: split[1].to_owned(),
        })
    }
}

impl TryFrom<&str> for CompoundPattern {
    type Error = ParseErrorKind;

    fn try_from(value: &str) -> Result<Self, ParseErrorKind> {
        let caps = RE_COMPOUND_PATTERN
            .captures(value)
            .ok_or(ParseErrorKind::CompoundPattern)?;
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
    type Error = ParseErrorKind;

    /// Format: `COMPOUNDSYLLABLE count vowels`
    fn try_from(value: &str) -> Result<Self, ParseErrorKind> {
        let split: Vec<_> = value.split_whitespace().collect();
        if split.len() != 2 {
            return Err(ParseErrorKind::CompoundSyllableCount(split.len()));
        }
        let to_parse = split[0];
        let count: u16 = to_parse
            .parse()
            .map_err(ParseErrorKind::CompoundSyllableParse)?;
        Ok(Self {
            count,
            vowels: split[1].to_owned(),
        })
    }
}

impl From<&str> for PartOfSpeech {
    #[inline]
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "noun" => Self::Noun,
            "verb" => Self::Verb,
            "adjective" => Self::Adjective,
            "determiner" => Self::Determiner,
            "adverb" => Self::Adverb,
            "pronoun" => Self::Pronoun,
            "preposition" => Self::Preposition,
            "conjunction" => Self::Conjunction,
            "interjection" => Self::Interjection,
            val => Self::Other(val.into()),
        }
    }
}

impl fmt::Display for PartOfSpeech {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PartOfSpeech::Noun => write!(f, "noun"),
            PartOfSpeech::Verb => write!(f, "verb"),
            PartOfSpeech::Adjective => write!(f, "adjective"),
            PartOfSpeech::Determiner => write!(f, "determiner"),
            PartOfSpeech::Adverb => write!(f, "adverb"),
            PartOfSpeech::Pronoun => write!(f, "pronoun"),
            PartOfSpeech::Preposition => write!(f, "preposition"),
            PartOfSpeech::Conjunction => write!(f, "conjunction"),
            PartOfSpeech::Interjection => write!(f, "interjection"),
            PartOfSpeech::Other(v) => write!(f, "{v}"),
        }
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
    #[inline]
    fn default() -> Self {
        Self::Utf8
    }
}
