use std::fmt::Display;
use std::num::ParseIntError;

use crate::helpers::convertu32;

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    Parse(ParseError),
    /// Error during building
    Build(BuildError),
    /// Regex error from user-provided input
    Regex(regex::Error),
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub struct ParseError {
    err: Box<ParseErrorType>,
    span: Option<Span>,
    ctx: String,
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    start: LineCol,
    end: LineCol,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineCol {
    line: u32,
    col: u32,
}

/// Errors related to [`DictBuilder`]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildError {
    /// Config specified twice
    CfgSpecTwice,
    CfgUnspecified,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ParseErrorType {
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
    FlagType,
    CompoundSyllableCount(usize),
    CompoundSyllableParse(ParseIntError),
    CompoundPattern,
    Phonetic(usize),
    PartOfSpeech(String),
    DictEntry,
    /// Regex error while parsing
    Regex(regex::Error),
}

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
    pub fn err(&self) -> &ParseErrorType {
        &self.err
    }
    pub(crate) fn span(&self) -> Option<&Span> {
        self.span.as_ref()
    }

    #[inline]
    pub(crate) fn new<T>(err: ParseErrorType, ctx: &str, line: T, col: T) -> Self
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
    pub(crate) fn new_nospan(err: ParseErrorType, ctx: &str) -> Self {
        Self::new(err, ctx, 0, 0)
    }

    #[inline]
    pub(crate) fn new_nocol<T>(err: ParseErrorType, ctx: &str, line: T) -> Self
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
        }

        self
    }
}

impl ParseErrorType {
    fn help_msg(&self) -> Option<&'static str> {
        match self {
            ParseErrorType::Boolean => {
                Some("boolean types cannot have anything else on their line")
            }
            ParseErrorType::Int(e) => todo!(),
            ParseErrorType::TableCount { expected, actual } => todo!(),
            _ => None,
        }
    }
}

/* trait impls */

impl Display for Error {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<ParseError> for Error {
    #[inline]
    fn from(value: ParseError) -> Self {
        Self::Parse(value)
    }
}

impl From<regex::Error> for Error {
    #[inline]
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}

impl From<regex::Error> for ParseErrorType {
    #[inline]
    fn from(value: regex::Error) -> Self {
        Self::Regex(value)
    }
}

impl From<ParseIntError> for ParseErrorType {
    #[inline]
    fn from(value: ParseIntError) -> Self {
        Self::Int(value)
    }
}

// impl Eq for Error {}

impl std::error::Error for Error {}

impl Display for ParseErrorType {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorType::Boolean => write!(f, "expected a boolean flag with no content"),
            ParseErrorType::Char(a, b) => write!(f, "expected {a} characters but got {b}"),
            ParseErrorType::Int(e) => write!(f, "failed to parse integer: {e}"),
            ParseErrorType::TableCount {
                expected,
                actual: received,
            } => {
                write!(f, "expected {expected} values in table but got {received}")
            }
            ParseErrorType::AffixHeader => write!(f, "could not parse affix header"),
            ParseErrorType::AffixBody => write!(f, "could not parse affix body"),
            ParseErrorType::AffixFlagMismatch(flag) => write!(
                f,
                "invalid affix body: flag does not match expected '{flag}'"
            ),
            ParseErrorType::AffixCrossProduct => {
                write!(f, "value is not a valid cross product indicator")
            }
            ParseErrorType::NonWhitespace(c) => write!(
                f,
                "unexpected non-comment characters before line termination starting at '{c}'"
            ),
            ParseErrorType::MorphInfoDelim(s) => {
                write!(f, "missing ':' delimiter in morph info at '{s}'")
            }
            ParseErrorType::MorphInvalidTag(s) => {
                write!(f, "tag '{s}' does not match any morphographic types")
            }
            ParseErrorType::ContainsWhitespace => write!(f, "not allowed to contain whitespace"),
            ParseErrorType::Encoding => write!(f, "unrecognized encoding"),
            ParseErrorType::FlagType => write!(f, "unrecognized flag"),
            ParseErrorType::CompoundPattern => write!(f, "invalid compound pattern"),
            ParseErrorType::Phonetic(n) => write!(f, "expected 2 items but got {n}"),
            ParseErrorType::DictEntry => write!(f, "invalid dictionary entry"),
            ParseErrorType::PartOfSpeech(s) => {
                write!(f, "value '{s}' is not a known part of speech")
            }
            ParseErrorType::ConversionSplit(_) => todo!(),
            ParseErrorType::CompoundSyllableCount(n) => write!(f, "expected 2 items but got {n}"),
            ParseErrorType::CompoundSyllableParse(e) => write!(f, "unable to parse integer: {e}"),
            ParseErrorType::Regex(e) => e.fmt(f),
            ParseErrorType::AffixHeader => todo!(),
        };
        Ok(())
    }
}

impl Display for ParseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.span {
            Some(span) => write!(f, "parse error at line {}: {}", span.start.line, self.err),
            None => write!(f, "error: {}", self.err),
        };
        Ok(())
    }
}
