use std::num;
use thiserror::Error;
// use super::spellcheck::affix_

#[derive(Error, Debug, PartialEq)]
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
    // #[error("the data for key `{0}` is not available")]

    // #[error("unknown data store error")]
    // Unknown,
}

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("missing root word {rootword}")]
    MissingRootWord { rootword: String }, // #[error("the data for key `{0}` is not available")]

                                          // #[error("unknown data store error")]
                                          // Unknown,
}
