use std::fmt::Display;
use std::num::ParseIntError;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Parse(ParseError),
    Build(BuildError),
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    err: Box<ParseErrorType>,
    span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    line: u32,
    col: u32,
}

/// Errors related to [`DictBuilder`]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// Config specified twice
    CfgSpecTwice,
    CfgUnspecified,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseErrorType {
    /// A boolean flag
    Boolean {
        s: String,
        key: String,
    },
    Char {
        s: String,
        count: usize,
    },
    Int {
        s: String,
        err: ParseIntError,
    },
    TableInt {
        s: String,
        err: ParseIntError,
    },
    TableCount {
        expected: u32,
        received: u32,
    },
    AffixHeader(String),
    AffixBody(String),
    AffixFlagMismatch {
        s: String,
        flag: String,
    },
    AffixCrossProduct(String),
    NonWhitespace(char),
    /// Should not contain whitespace but does
    ContainsWhitespace(String),
    /// Missing the ':' delimiter
    MorphInfoDelim(String),
    /// Unrecognized morph info tag
    MorphInvalidTag(String),
    Conversion {
        s: String,
        count: usize,
    },
    Encoding(String),
    Flag(String),
    CompoundSyllable(String),
    CompoundPattern(String),
    Phonetic(String),
    PartOfSpeech(String),
    CharCount {
        s: String,
        expected: u32,
    },
    DictEntry(String),
}

impl Span {
    pub(crate) fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

impl ParseError {
    pub fn err(&self) -> &ParseErrorType {
        &self.err
    }
    pub fn span(&self) -> &Span {
        &self.span
    }

    #[inline]
    pub(crate) fn new_nospan(err: ParseErrorType) -> Self {
        Self {
            err: Box::new(err),
            span: Span { line: 0, col: 0 },
        }
    }

    #[inline]
    pub(crate) fn new(err: ParseErrorType, line: u32, col: u32) -> Self {
        Self {
            err: Box::new(err),
            span: Span { line, col },
        }
    }

    pub(crate) const fn add_offset_ret(mut self, line: u32, col: u32) -> Self {
        self.span.line += line;
        self.span.col += col;
        self
    }
}

impl ParseErrorType {
    pub(crate) fn new_bool(key: &str, s: &str) -> Self {
        Self::Boolean {
            key: key.to_owned(),
            s: s.to_owned(),
        }
    }

    pub(crate) fn new_char(count: usize, s: &str) -> Self {
        Self::Char {
            count,
            s: s.to_owned(),
        }
    }

    pub(crate) fn new_int(s: &str, err: ParseIntError) -> Self {
        Self::Int {
            s: s.to_owned(),
            err,
        }
    }
    pub(crate) fn new_table_int(s: &str, err: ParseIntError) -> Self {
        Self::TableInt {
            s: s.to_owned(),
            err,
        }
    }
    pub(crate) const fn new_table_count(expected: u32, received: u32) -> Self {
        Self::TableCount { expected, received }
    }

    fn help_msg(&self) -> Option<&'static str> {
        match self {
            ParseErrorType::Boolean { .. } => {
                Some("boolean types cannot have anything else on their line")
            }
            ParseErrorType::Char { s, count } => todo!(),
            ParseErrorType::Int { s, err } => todo!(),
            ParseErrorType::TableInt { s, err } => todo!(),
            ParseErrorType::TableCount { expected, received } => todo!(),
            ParseErrorType::AffixHeader(_) => todo!(),
            ParseErrorType::AffixBody(_) => todo!(),
            ParseErrorType::AffixFlagMismatch { s, flag } => todo!(),
            ParseErrorType::AffixCrossProduct(_) => todo!(),
            ParseErrorType::NonWhitespace(_) => todo!(),
            ParseErrorType::ContainsWhitespace(_) => todo!(),
            ParseErrorType::MorphInfoDelim(_) => todo!(),
            ParseErrorType::Conversion { s, count } => todo!(),
            ParseErrorType::Encoding(_) => todo!(),
            ParseErrorType::Flag(_) => todo!(),
            ParseErrorType::CompoundSyllable(_) => todo!(),
            ParseErrorType::CompoundPattern(_) => todo!(),
            ParseErrorType::Phonetic(_) => todo!(),
            ParseErrorType::CharCount { s, expected } => todo!(),
            ParseErrorType::DictEntry(_) => todo!(),
            _ => None,
        }
    }
}

/* trait impls */

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<ParseErrorType> for ParseError {
    /// Default to a non spanned error
    fn from(value: ParseErrorType) -> Self {
        Self::new_nospan(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl std::error::Error for Error {}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorType::Boolean { s, key } => write!(
                f,
                "{key} is a boolean flag; got content '{s}'"
            ),
            ParseErrorType::Char { s, count } => write!(
                f,
                "expected a single character flag but got {count} chars at '{s}'"
            ),
            ParseErrorType::Int { s, err } => write!(f, "failed to parse integer at '{s}': {err}"),
            ParseErrorType::TableInt { s, err } => {
                write!(f, "failed to parse table item count at '{s}': {err}")
            }
            ParseErrorType::TableCount { expected, received } => {
                write!(f, "expected {expected} values in table but got {received}")
            }
            ParseErrorType::AffixHeader(s) => write!(f, "could not parse affix header pattern at '{s}'"),
            ParseErrorType::AffixBody(s) => write!(f, "could not parse affix body pattern at '{s}'"),
            ParseErrorType::AffixFlagMismatch { s, flag } => write!(f, "could not parse affix body pattern at '{s}': flag does not match expected '{flag}'"),
            ParseErrorType::AffixCrossProduct(s) => write!(f, "value {s} is not a valid cross product indicator"),
            ParseErrorType::NonWhitespace(c) => write!(f, "unexpected non-comment characters before line termination starting at '{c}'"),
            ParseErrorType::MorphInfoDelim(s) => write!(f, "missing ':' delimiter in morph info at '{s}'"),
            ParseErrorType::MorphInvalidTag(s) => write!(f, "tag '{s}' does not match any morphographic types"),
            ParseErrorType::ContainsWhitespace(_) => todo!(),
            ParseErrorType::Conversion { s, count } => todo!(),
            ParseErrorType::Encoding(_) => todo!(),
            ParseErrorType::Flag(_) => todo!(),
            ParseErrorType::CompoundSyllable(_) => todo!(),
            ParseErrorType::CompoundPattern(_) => todo!(),
            ParseErrorType::Phonetic(_) => todo!(),
            ParseErrorType::CharCount { s, expected } => todo!(),
            ParseErrorType::DictEntry(_) => todo!(),
            ParseErrorType::PartOfSpeech(s) => write!(f, "value '{s}' is not a known part of speech"),
        };
        Ok(())
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error at line {}: {}", self.span.line, self.err);
        Ok(())
    }
}
