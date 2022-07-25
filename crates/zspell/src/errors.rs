use std::{io, num};
use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug, PartialEq, Eq)]
pub enum AffixError {
    #[error("bad number at")]
    NumParse(#[from] num::ParseIntError),

    // #[error("Bad number at {val}")]
    // TokenTypeError(TokenType)
    #[error("token type error")]
    WrongTokenType,

    #[error("no conversion input found")]
    NoConversionInput,

    #[error("no conversion output found")]
    NoConversionOutput,

    #[error("bad affix syntax at {0}")]
    Syntax(String),

    #[error("expected {missing} more items of type {expected}; received {received}")]
    TableCount {
        expected: String,
        received: String,
        missing: u16,
    },

    #[error("no identifier found")]
    MissingIdentifier,

    #[error("bad or missing cross product info")]
    BadCrossProduct,

    #[error("bad token type")]
    BadTokenType,

    #[error("bad encoding type specified")]
    BadEncodingType,
}

/// Error while creating the dictionary
#[non_exhaustive]
#[derive(Error, Debug, PartialEq, Eq)]
pub enum DictError {
    #[error("dictionary needs to be compiled before use")]
    NotCompiled,

    #[error("misc error")]
    Other,

    #[error("issue with file {fname}: {orig_e}")]
    FileError {
        fname: String,
        orig_e: io::ErrorKind,
    },
}

/// Error while compiling the dictionary
#[non_exhaustive]
#[derive(Error, Debug, PartialEq, Eq)]
pub enum CompileError {
    #[error("missing root word {rootword}")]
    MissingRootWord { rootword: String },
}
