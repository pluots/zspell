//! Crate error types (module is semi-unstable)
//!
//! [`Error`] is the main error type for this crate, all other types of errors
//! will fall under it.

use core::prelude::v1;
use std::fmt::Display;
use std::num::ParseIntError;

use crate::affix::FlagType;
use crate::dict::FlagValue;
use crate::helpers::convertu32;

/// Main crate error type, returned by most public functions
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// Error during parsing
    Parse(ParseError),
    /// Error during building
    Build(BuildError),
    /// Regex error from user-provided input
    Regex(regex::Error),

    Io(IoError),
}

/// An error that occured while parsing, consisting of an error variant and a
/// location
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    /// The error that occured
    err: Box<ParseErrorKind>,
    /// Approximate location of the error in source
    span: Option<Span>,
    /// Context of what caused this error
    ctx: String,
}

/// A representation of where a [`ParseError`] occured
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    start: LineCol,
    end: LineCol,
}

/// A location within a file
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCol {
    line: u32,
    col: u32,
}

/// Errors that can occur when building a dictionary
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    /// Config specified twice in the builder
    BuilderCfgSpecTwice,
    /// Builder config was not specified
    BuilderCfgUnspecified,
    /// A given flag is invalid
    UnknownFlag(String),
    /// Got a flag that does not match the given type
    FlagType { value: String, expected: FlagType },
    /// A flag was used for >1 thing in an affix file
    DuplicateFlag {
        /// Flag string value
        flag: String,
        /// Initial duplicate type
        t1: FlagValue,
        /// Second duplicate type if Some; affix rule if None
        t2: Option<FlagValue>,
    },
    /// A flag in a dictionary file does not match any known flags
    NonmatchingFlag { stem: String, flag: String },
}

/// An I/O error. This is a wrapper around [`std::io::ErrorKind`]
#[derive(Clone, Debug, PartialEq)]
pub struct IoError {
    fname: String,
    err: std::io::ErrorKind,
}

/// A kind of error that would occur during parsing, with additional information
#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    /// A boolean flag
    Boolean,
    /// Expected `a` chars but got `b`
    Char(usize, usize),
    /// Error parsing any integer
    Int(ParseIntError),
    /// Wrong number of items in a table
    TableCount {
        expected: u32,
        actual: u32,
    },
    AffixHeader,
    AffixBody,
    /// String holds the expected flag
    AffixFlagMismatch(String),
    AffixCrossProduct,
    NonWhitespace(char),
    /// Should not contain whitespace but does
    ContainsWhitespace,
    /// Missing the ':' delimiter
    MorphInfoDelim(String),
    /// Unrecognized morph info tag
    MorphInvalidTag(String),
    /// Expected a conversion with two items to split but got this many
    ConversionSplit(usize),
    Encoding,
    /// Failure trying to parse `FLAG`
    FlagType,
    FlagParse(FlagType),
    /// Up to 4 ascii characters max, alphanumeric
    InvalidFlag,

    CompoundSyllableCount(usize),
    CompoundSyllableParse(ParseIntError),
    // An error parsing the personal dictionary
    Personal,
    CompoundPattern,
    Phonetic(usize),
    PartOfSpeech(String),
    DictEntry,
    /// Regex error while parsing
    Regex(regex::Error),
}

/// An error returned from functions that expect a word to be present in a
/// dictionary but were unable to find the word.
#[derive(Clone, Debug, PartialEq)]
pub struct WordNotFoundError;

impl Span {
    /// New with only start line & column specified. End will be start line + 1
    pub(crate) fn new(line: u32, col: u32) -> Self {
        let lc = LineCol { line, col };
        Self {
            start: lc,
            end: LineCol {
                line: line + 1,
                col,
            },
        }
    }
}

impl ParseError {
    #[inline]
    pub fn err(&self) -> &ParseErrorKind {
        &self.err
    }

    #[inline]
    pub fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    #[inline]
    pub(crate) fn new<T>(err: ParseErrorKind, ctx: &str, line: T, col: T) -> Self
    where
        T: TryInto<u32> + Display + Copy,
    {
        Self {
            err: Box::new(err),
            span: Some(Span::new(convertu32(line), convertu32(col))),
            ctx: ctx.to_owned(),
        }
    }

    #[inline]
    pub(crate) fn new_nospan(err: ParseErrorKind, ctx: &str) -> Self {
        Self {
            err: Box::new(err),
            span: None,
            ctx: ctx.to_owned(),
        }
    }

    #[inline]
    pub(crate) fn new_nocol<T>(err: ParseErrorKind, ctx: &str, line: T) -> Self
    where
        T: TryInto<u32> + Display + Copy,
    {
        Self::new(err, ctx, convertu32(line), 0)
    }

    pub(crate) fn add_offset_ret<T>(mut self, line: T, col: T) -> Self
    where
        T: TryInto<u32> + Display + Copy,
    {
        let l_inc = convertu32(line);
        let c_inc = convertu32(col);

        if let Some(span) = self.span.as_mut() {
            span.start.line += l_inc;
            span.end.line += l_inc;
            span.start.col += c_inc;
            span.end.col += c_inc;
        } else {
            self.span = Some(Span::new(l_inc, c_inc));
        }

        self
    }
}

impl ParseErrorKind {
    fn help_msg(&self) -> Option<&'static str> {
        match self {
            ParseErrorKind::Boolean => {
                Some("boolean types cannot have anything else on their line")
            }
            ParseErrorKind::Int(e) => todo!(),
            ParseErrorKind::TableCount { expected, actual } => todo!(),
            _ => None,
        }
    }
}

impl IoError {
    pub(crate) fn new(fname: &str, err: std::io::ErrorKind) -> Self {
        Self {
            fname: fname.to_owned(),
            err,
        }
    }
}

/* trait impls */

impl std::error::Error for Error {}
impl std::error::Error for ParseError {}
impl std::error::Error for ParseErrorKind {}
impl std::error::Error for BuildError {}

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {e}"),
            Error::Build(e) => write!(f, "build error: {e}"),
            Error::Regex(e) => write!(f, "regex error: {e}"),
            Error::Io(e) => write!(f, "io error: {e}"),
        }
    }
}

impl Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.span {
            Some(span) => write!(
                f,
                "parse error at line {}: {}. Context: '{}'",
                span.start.line, self.err, self.ctx
            )?,
            None => write!(f, "parse error: {}", self.err)?,
        };
        Ok(())
    }
}

impl Display for ParseErrorKind {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorKind::Boolean => write!(f, "expected a boolean flag with no content"),
            ParseErrorKind::Char(a, b) => write!(f, "expected {a} flags but got {b}"),
            ParseErrorKind::Int(e) => write!(f, "failed to parse integer: {e}"),
            ParseErrorKind::TableCount {
                expected,
                actual: received,
            } => write!(f, "expected {expected} values in table but got {received}"),
            ParseErrorKind::AffixHeader => write!(f, "could not parse affix header"),
            ParseErrorKind::AffixBody => write!(f, "could not parse affix body"),
            ParseErrorKind::AffixFlagMismatch(flag) => write!(
                f,
                "invalid affix body: flag does not match expected '{flag}'"
            ),
            ParseErrorKind::AffixCrossProduct => {
                write!(f, "value is not a valid cross product indicator")
            }
            ParseErrorKind::NonWhitespace(c) => write!(
                f,
                "unexpected non-comment characters before line termination starting at '{c}'"
            ),
            ParseErrorKind::MorphInfoDelim(s) => {
                write!(f, "missing ':' delimiter in morph info at '{s}'")
            }
            ParseErrorKind::MorphInvalidTag(s) => {
                write!(f, "tag '{s}' does not match any morphographic types")
            }
            ParseErrorKind::ContainsWhitespace => write!(f, "not allowed to contain whitespace"),
            ParseErrorKind::Encoding => write!(f, "unrecognized encoding"),
            ParseErrorKind::FlagType => write!(f, "unrecognized flag"),
            ParseErrorKind::CompoundPattern => write!(f, "invalid compound pattern"),
            ParseErrorKind::Phonetic(n) => write!(f, "expected 2 items but got {n}"),
            ParseErrorKind::DictEntry => write!(f, "invalid dictionary entry"),
            ParseErrorKind::PartOfSpeech(s) => {
                write!(f, "value '{s}' is not a known part of speech")
            }
            ParseErrorKind::ConversionSplit(n) => {
                write!(f, "expected a conversion with 2 items but got {n}")
            }
            ParseErrorKind::CompoundSyllableCount(n) => write!(f, "expected 2 items but got {n}"),
            ParseErrorKind::CompoundSyllableParse(e) => write!(f, "unable to parse integer: {e}"),
            ParseErrorKind::Regex(e) => e.fmt(f),
            ParseErrorKind::AffixHeader => todo!(),
            ParseErrorKind::Personal => write!(f, "error parsing entry in personal dictionary"),
            ParseErrorKind::InvalidFlag => {
                write!(f, "expected a single alphanumeric flag (4 bytes maximum)")
            }
            ParseErrorKind::FlagParse(v) => write!(f, "error parsing flag of type '{v}'"),
        }
    }
}

impl Display for BuildError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::BuilderCfgSpecTwice => {
                write!(f, "configuration specified twice in builder")
            }
            BuildError::BuilderCfgUnspecified => {
                write!(f, "configuration unspecified twice in builder")
            }
            BuildError::UnknownFlag(v) => write!(
                f,
                "got flag `{v}` that wasn't present in affix configuration"
            ),
            BuildError::FlagType { value, expected } => {
                write!(f, "value '{value}' is not valid for flag type {expected}")
            }
            BuildError::DuplicateFlag {
                flag,
                t1,
                t2: Some(v),
            } => write!(
                f,
                "flag '{flag}' used for two or more flags: '{t1}' and '{v}'"
            ),
            BuildError::DuplicateFlag { flag, t1, t2: None } => write!(
                f,
                "flag '{flag}' used for two or more flags: '{t1}' and affix rule"
            ),
            BuildError::NonmatchingFlag { stem, flag } => write!(
                f,
                "stem '{stem}' is marked with flag '{flag}' but it does not match any patterns"
            ),
        }
    }
}

impl Display for IoError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in file '{}' {}", self.fname, self.err)
    }
}

impl From<ParseError> for Error {
    #[inline]
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<BuildError> for Error {
    #[inline]
    fn from(value: BuildError) -> Self {
        Self::Build(value)
    }
}

impl From<regex::Error> for Error {
    #[inline]
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}

impl From<regex::Error> for ParseErrorKind {
    #[inline]
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}

impl From<ParseIntError> for ParseErrorKind {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        Self::Int(value)
    }
}

impl From<IoError> for Error {
    #[inline]
    fn from(value: IoError) -> Self {
        Self::Io(value)
    }
}
