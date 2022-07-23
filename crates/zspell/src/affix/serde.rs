//! Affix Ser/Des module
//!
//! This module handles loading in an affix file to an [`AffixConfig`] object.
//! Usually it is not accessed directly.

use std::convert::TryFrom;

use strum::{EnumProperty, VariantNames};
use unicode_segmentation::UnicodeSegmentation;

use crate::affix::Config;
use crate::affix::{Conversion, EncodingType, Rule, TokenType};
use crate::errors::AffixError;
use crate::graph_vec;
use crate::unwrap_or_ret_e;

/// Unwrap a [`TokenData`] type
macro_rules! t_data_unwrap {
    ( $token:ident, $field:ident ) => {
        match $token.data {
            ProcessedTokenData::$field(f) => f,
            _ => return Err(AffixError::BadTokenType),
        }
    };
}

pub(crate) use t_data_unwrap;

macro_rules! parentify {
    // Boolean field just assigns true and returns Ok (Flag is just either there
    // or not)
    ( $parent:ident.$field:ident, $token:ident, bool ) => {
        match $token.data {
            ProcessedTokenData::Bool(b) => $parent.$field = b,
            _ => return Err(AffixError::BadTokenType),
        }
    };
    ( $parent:ident.$field:ident, $token:ident, int ) => {
        match $token.data {
            ProcessedTokenData::Int(b) => $parent.$field = b,
            _ => return Err(AffixError::BadTokenType),
        }
    };
    ( $parent:ident.$field:ident, $token:ident, str_replace ) => {
        match $token.data {
            ProcessedTokenData::String(s) => $parent.$field = s.to_owned(),
            _ => return Err(AffixError::BadTokenType),
        }
    };

    // Use str_add any time we have a `String` that we want to append to.
    // Basically the same as above except we append to the existing vector and
    // sort rather than replacing what's there. Usable for `Vec<&str>`.
    ( $parent:ident.$field:ident, $token:ident, str_append ) => {
        match $token.data {
            ProcessedTokenData::String(s) => {
                let mut tmp = graph_vec!(s);
                $parent.$field.append(&mut tmp);
                $parent.$field.sort();
                $parent.$field.dedup();
            }
            _ => return Err(AffixError::BadTokenType),
        }
    };
}

/// Populate an Affix class from the string version of a file. This is the main
/// function exported from this module. `ax` is the [`AffixConfig`] object to
/// load, `s` is the file raw string to load in
///
/// # Errors
///
/// Returns an [`AffixError`] if there is any problem loading the file
#[inline]
pub fn load_affix_from_str(ax: &mut Config, s: &str) -> Result<(), AffixError> {
    // This will give us a list of tokens with no processing applied
    let raw_stripped = strip_comments(s);
    let raw_str = raw_stripped.as_str();
    let raw_tokens = create_raw_tokens(raw_str);

    let tokens = create_processed_tokens(raw_tokens)?;
    set_parent(ax, tokens)
}

/// Strip "#" comments from a &str. Breaks from a found "#" to the next newline;
/// does not remove the newlines
fn strip_comments(s: &str) -> String {
    let mut newstr = String::new();
    let mut paused = false;
    // Logic to skip from # to newline
    for ch in s.chars() {
        if paused && ch == '\n' {
            paused = false;
        } else if paused {
            continue;
        } else if ch == '#' {
            paused = true;
            continue;
        }
        newstr.push(ch);
    }
    newstr
}

/// A token for a specific affix option
/// Holds a type and the stream of words that follow
/// Words have been split on whitespace and the token prefix has been stripped
#[derive(Debug, PartialEq)]
struct AffixRawToken<'a> {
    pub ttype: TokenType,
    pub content: Vec<&'a str>,
}

impl AffixRawToken<'_> {
    pub fn new(ttype: TokenType, content: Vec<&str>) -> AffixRawToken {
        AffixRawToken { ttype, content }
    }
}

/// Create a list of tokens with matched type and string content; do not do
/// anything to analyze them
///
/// Note: this strips prefixes
fn create_raw_tokens<'a>(s: &'a str) -> Vec<AffixRawToken<'a>> {
    // Temporarially hold the "next vector" rather than working one, blank until
    // needed
    let mut working_vec: Vec<&'a str> = Vec::new();
    let mut working_type = TokenType::FileStart;

    let mut ret: Vec<AffixRawToken> = Vec::new();

    // Each token is a "word", split by UTF-8 boundaries
    for word in s.split_whitespace() {
        if TokenType::VARIANTS.contains(&word) {
            // Push this token...
            ret.push(AffixRawToken::new(working_type, working_vec));
            // ...And start a new one
            working_vec = Vec::new();
            working_type = TokenType::try_from(word).unwrap();
        } else {
            // Not at a delimiting key; just add this to our current token
            working_vec.push(word);
        }
    }

    ret.push(AffixRawToken::new(working_type, working_vec));

    ret
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessedTokenData<'a> {
    Bool(bool),
    Int(u16),
    String(&'a str),
    // Splits each line by whitespace, then again by lines
    Table(Vec<Vec<&'a str>>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ProcessedToken<'a> {
    pub ttype: TokenType,
    pub data: ProcessedTokenData<'a>,
}

/// Use the first raw token to determine how many more to read into the table
/// Returns a u16 if successful, error otherwise
fn get_table_item_count(token: &AffixRawToken) -> Result<u16, AffixError> {
    // Recall: our tokens have the token prefix stripped

    // count_str gives us the string that should represent the table's item count
    let count_str = match token.ttype {
        // AF [n]
        TokenType::AffixFlag => token.content.first(),
        // AM [n]
        TokenType::MorphAlias => token.content.first(),
        // REP [n]
        TokenType::Replacement => token.content.first(),
        // MAP [n]
        TokenType::Mapping => token.content.first(),
        // PHONE [n]
        TokenType::Phonetic => token.content.first(),
        // BREAK [n]
        TokenType::Breakpoint => token.content.first(),
        // COMPOUNDRULE [n]
        TokenType::CompoundRule => token.content.first(),
        // CHECKCOMPOUNDPATTERN [n]
        TokenType::CompoundForbidPatterns => token.content.first(),
        // PFX flag cross_product number
        TokenType::Prefix => token.content.get(2),
        // SFX flag cross_product number
        TokenType::Suffix => token.content.get(2),
        // ICONV [n]
        TokenType::AffixInputConversion => token.content.first(),
        // OCONV [n]
        TokenType::AffixOutputConversion => token.content.first(),
        // Any tokens types that don't have a table
        _ => return Ok(0),
    };

    let to_parse = unwrap_or_ret_e!(count_str, AffixError::Syntax(token.content.join(" ")));
    match to_parse.parse() {
        Ok(v) => Ok(v),
        Err(e) => Err(AffixError::NumParse(e)),
    }
}

/// Loop through a vector of raw tokens and create the processed version
fn create_processed_tokens(tokens: Vec<AffixRawToken>) -> Result<Vec<ProcessedToken>, AffixError> {
    // Vector to hold what we will return
    let mut retvec = Vec::new();
    // If we need to accumulate values for a table, use these items
    let mut table_accum_count = 0u16;
    let mut table_accum_type = TokenType::FileStart; // FileStart is kind of a dummy token
    let mut table_accum_vec = Vec::new();

    for token in tokens {
        // FileStart is just a dummy token
        if token.ttype == TokenType::FileStart {
            continue;
        }

        // Check accumulate logic first
        if table_accum_count > 0 {
            // If we ar eaccumulating, just push this token and go on to the next
            // one to our temp working vector
            // Validate token type first
            if token.ttype != table_accum_type {
                return Err(AffixError::TableCount {
                    expected: table_accum_type.to_string(),
                    received: token.ttype.to_string(),
                    missing: table_accum_count,
                });
            }

            table_accum_vec.push(token.content);
            table_accum_count -= 1;
            if table_accum_count > 0 {
                continue;
            }

            // If we are here, that means we are on our last token.
            // Finalie it to the return vec
            retvec.push(ProcessedToken {
                ttype: token.ttype,
                data: ProcessedTokenData::Table(table_accum_vec),
            });
            // And reset our status
            table_accum_vec = Vec::new();
            continue;
        }

        // If we're not accumulating, just match based on token type
        match token.ttype.get_str("dtype").unwrap() {
            // String or bool are straightforward
            "str" => {
                if token.content.len() != 1 {
                    return Err(AffixError::Syntax(token.ttype.to_string()));
                };
                retvec.push(ProcessedToken {
                    ttype: token.ttype,
                    data: ProcessedTokenData::String(token.content[0]),
                });
            }
            "bool" => {
                if !token.content.is_empty() {
                    return Err(AffixError::Syntax(token.ttype.to_string()));
                };
                retvec.push(ProcessedToken {
                    ttype: token.ttype,
                    data: ProcessedTokenData::Bool(true),
                });
            }
            "int" => {
                if token.content.len() != 1 {
                    return Err(AffixError::Syntax(token.ttype.to_string()));
                };
                let val = token
                    .content
                    .first()
                    .expect("Issue getting token content")
                    .parse()?;
                retvec.push(ProcessedToken {
                    ttype: token.ttype,
                    data: ProcessedTokenData::Int(val),
                });
            }
            // For table - figure out item count, push this token,
            "table" => {
                table_accum_count = match get_table_item_count(&token) {
                    Ok(val) => val,
                    Err(s) => return Err(s),
                };
                table_accum_vec.push(token.content);
                table_accum_type = token.ttype;
            }
            _ => panic!("Unexpected token type"),
        }
    }

    Ok(retvec)
}

// Actually go through and set the parent here
fn set_parent(ax: &mut Config, tokens: Vec<ProcessedToken>) -> Result<(), AffixError> {
    for token in tokens {
        match token.ttype {
            TokenType::Encoding => match EncodingType::try_from(t_data_unwrap!(token, String)) {
                Ok(et) => ax.encoding = et,
                Err(_) => return Err(AffixError::BadEncodingType),
            },
            TokenType::FlagType => todo!("Flag is not yet supported"),
            TokenType::ComplexPrefixes => parentify!(ax.complex_prefixes, token, bool),
            TokenType::Language => parentify!(ax.lang, token, str_replace),
            TokenType::IgnoreChars => parentify!(ax.ignore_chars, token, str_append),
            TokenType::AffixFlag => parentify!(ax.afx_flag_vector, token, str_append),
            // TokenType::MorphAlias => todo!(),
            // TokenType::NeighborKeys => todo!(), // DO NOT SORT
            TokenType::TryCharacters => parentify!(ax.try_characters, token, str_append),
            TokenType::NoSuggestFlag => parentify!(ax.nosuggest_flag, token, str_replace),
            // TokenType::CompoundSuggestionsMax => todo!(),
            // TokenType::NGramSuggestionsMax => todo!(),
            // TokenType::NGramDiffMax => todo!(),
            // TokenType::NGramLimitToDiffMax => todo!(),
            // TokenType::NoSpaceSubs => todo!(),
            // TokenType::KeepTerminationDots => todo!(),
            TokenType::Replacement => {
                ax.replacements = Conversion::from_processed_token(token, false)?;
            }

            // TokenType::Mapping => todo!(),
            // TokenType::Phonetic => todo!(),
            // TokenType::WarnRareFlag => todo!(),
            // TokenType::ForbidWarnWords => todo!(),
            // TokenType::Breakpoint => todo!(),
            // TokenType::CompoundRule => todo!(),
            TokenType::CompoundMinLength => parentify!(ax.compound_min_length, token, int),
            // TokenType::CompoundFlag => todo!(),
            // TokenType::CompoundBeginFlag => todo!(),
            // TokenType::CompoundEndFlag => todo!(),
            // TokenType::CompoundMiddleFlag => todo!(),
            // TokenType::CompoundOnlyFlag => todo!(),
            // TokenType::CompoundPermitFlag => todo!(),
            // TokenType::CompoundForbidFlag => todo!(),
            // TokenType::CompoundMoreSuffixes => todo!(),
            // TokenType::CompoundRoot => todo!(),
            // TokenType::CompoundWordMax => todo!(),
            // TokenType::CompoundForbidDuplication => todo!(),
            // TokenType::CompoundForbidRepeat => todo!(),
            // TokenType::CompoundForbidUpperBoundary => todo!(),
            // TokenType::CompoundForbidTriple => todo!(),
            // TokenType::CompoundSimplifyTriple => todo!(),
            // TokenType::CompoundForbidPatterns => todo!(),
            // TokenType::CompoundForceUpper => todo!(),
            // TokenType::CompoundForceSyllable => todo!(),
            // TokenType::CompoundSyllableNumber => todo!(),
            TokenType::Prefix => ax.affix_rules.push(Rule::from_processed_token(token)?),
            TokenType::Suffix => ax.affix_rules.push(Rule::from_processed_token(token)?),
            // TokenType::AffixCircumfixFlag => todo!(),
            // TokenType::AffixForbiddenWordFlag => todo!(),
            // TokenType::AffixFullStrip => todo!(),
            // TokenType::AffixKeepCase => todo!(),
            TokenType::AffixInputConversion => {
                ax.input_conversions = Conversion::from_processed_token(token, false)?;
            }

            TokenType::AffixOutputConversion => {
                ax.output_conversions = Conversion::from_processed_token(token, false)?;
            }

            // TokenType::AffixLemmaPresentDeprecated => todo!(),
            // TokenType::AffixNeededFlag => todo!(),
            // TokenType::AffixPseudoRootFlagDeprecated => todo!(),
            // TokenType::AffixSubstandardFlag => todo!(),
            // TokenType::AffixWordChars => todo!(),
            // TokenType::AffixCheckSharps => todo!(),
            // TokenType::FileStart => todo!(),
            _ => (),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_strip_comments() {
        let s = "abc\ndef\n# comment\nline with # comment\nendline";
        assert_eq!(strip_comments(s), "abc\ndef\n\nline with \nendline");
    }

    #[test]
    fn test_create_raw_token() {
        let s = "SYLLABLENUM a b c PFX prefix CIRCUMFIX COMPOUNDLAST";
        let test_vec = vec![
            AffixRawToken::new(TokenType::FileStart, vec![]),
            AffixRawToken::new(TokenType::CompoundSyllableNumber, vec!["a", "b", "c"]),
            AffixRawToken::new(TokenType::Prefix, vec!["prefix"]),
            AffixRawToken::new(TokenType::AffixCircumfixFlag, vec![]),
            AffixRawToken::new(TokenType::CompoundEndFlag, vec![]),
        ];
        assert_eq!(create_raw_tokens(s), test_vec);
    }

    #[test]
    fn test_get_table_item_count() {
        let count_res0 = get_table_item_count(&AffixRawToken {
            ttype: TokenType::Mapping,
            content: vec!["1"],
        });
        assert_eq!(count_res0, Ok(1));

        let count_res1 = get_table_item_count(&AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["A", "Y", "6"],
        });
        assert_eq!(count_res1, Ok(6));

        let count_res2 = get_table_item_count(&AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["A", "Y"],
        });
        assert_eq!(count_res2, Err(AffixError::Syntax("A Y".into())));

        let count_res3 = get_table_item_count(&AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["A", "Y", "-80"],
        });
        // Can't figure out a good way to compare the errors
        match count_res3.unwrap_err() {
            AffixError::NumParse(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn test_create_processed_tokens() {
        // Test boolean
        let t0 = AffixRawToken {
            ttype: TokenType::NoSpaceSubs,
            content: vec![],
        };
        let res0 = Ok(vec![ProcessedToken {
            ttype: TokenType::NoSpaceSubs,
            data: ProcessedTokenData::Bool(true),
        }]);
        assert_eq!(create_processed_tokens(vec![t0]), res0);

        // Test string
        let t1 = AffixRawToken {
            ttype: TokenType::Language,
            content: vec!["mylanguage"],
        };
        let res1 = Ok(vec![ProcessedToken {
            ttype: TokenType::Language,
            data: ProcessedTokenData::String("mylanguage"),
        }]);
        assert_eq!(create_processed_tokens(vec![t1]), res1);

        // Test table creations
        let t20 = AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["V", "N", "2"],
        };
        let t21 = AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["V", "e"],
        };
        let t22 = AffixRawToken {
            ttype: TokenType::Prefix,
            content: vec!["V", "0"],
        };
        let res1 = Ok(vec![ProcessedToken {
            ttype: TokenType::Prefix,
            data: ProcessedTokenData::Table(vec![
                vec!["V", "N", "2"],
                vec!["V", "e"],
                vec!["V", "0"],
            ]),
        }]);
        assert_eq!(create_processed_tokens(vec![t20, t21, t22]), res1);
    }
}
