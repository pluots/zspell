//! Dictionary configuration options framework
//!
//! This module contains everything related to the configuration of a
//! [`crate::Dictionary`], which can then be used for checking words.
//!
//! Usually this configuration is loaded via an affix file (typically ending in
//! `.aff`), but it can also be created programatically if desired.
//!
//! This module should be considered unstable as its usage is finalized.

mod serde;
mod types;

use unicode_segmentation::UnicodeSegmentation;

use crate::errors::AffixError;
use crate::graph_vec;

use serde::t_data_unwrap;
pub use serde::{load_affix_from_str, ProcessedToken, ProcessedTokenData};
pub use types::{Conversion, EncodingType, Rule, RuleType, TokenType};

/// Dictionary configuration object that holds affix file data
///
/// This holds the entire contents of the affix file as an AST representation
/// and is intended to be used throughout program lifetime. If you are
/// uninterested in modifying an existing dictionary structure, you are likely
/// interested in just using [`crate::Dictionary`] and its methods.
///
/// # Internal working
///
/// Generally within this class, a "string" is represented as a Vec<String> or
/// Vec<&str>, i.e. a vector of string graphemes. This is because many languages
/// may require accurate unicode segmentation to work properly. It is not yet
/// understood whether this is the best practice, so this may change in the
/// future.
///
/// Any type that can be modified must be owned (e.g. String, Vec), others may
/// be borrowed.
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq)]
pub struct Config {
    /// Charset to use, reference to an [`EncodingType`] Currently this is
    /// unused; only UTF-8 is supported. However, the affix file must still have
    /// an accurate definition.
    pub encoding: EncodingType,

    /// Twofold prefix skipping for e.g. right-to-left languages
    pub complex_prefixes: bool,

    /// Language code, currently unused. Consider this unstable as it may change
    /// to be an object reference.
    pub lang: String,

    /// List of characters to ignore
    pub ignore_chars: Vec<String>,

    /// List of usable flag vectors. Defaults to all things after "/"" in a dict.
    pub afx_flag_vector: Vec<String>,

    // ## Suggestion-related items
    /// List of e.g. "qwerty", "asdfg" that define neighbors
    pub keys: Vec<Vec<String>>,

    /// Suggest words that differe by 1 try character
    pub try_characters: Vec<String>,

    /// Flag used to indicate words that should not be suggested
    pub nosuggest_flag: String,

    /// Maximum compound word suggestions
    pub compound_suggestions_max: u16,

    /// Max number of ngram suggestions
    pub ngram_suggestions_max: u16,

    /// N-gram similarity limit
    pub ngram_diff_max: u16,

    /// Remove all suggestions except the diff max
    pub ngram_limit_to_diff_max: bool,

    /// Don't suggest anything with spaces
    pub no_split_suggestions: bool,

    /// If a dot comes with the spellcheck, return one with a suggestion word
    pub keep_termination_dots: bool,

    /// Note rare (i.e. commonly misspelled) words with this flag
    pub warn_rare_flag: String,

    /// Whether to never suggest words with the warn flag (above)
    pub forbid_warn_words: bool,

    // pub replacements: Vec<&'a ReplaceRule<'a>>,
    // maps: Vec<>, // MAP
    // phones: Vec<>

    // ## Compounding-related items

    // break_points: Vec<>
    // compound_rules: Vec<>
    /// Minimum length of words used in a compound
    pub compound_min_length: u16,

    /// Words with this flag may be in compounds
    pub compound_flag: Option<String>,

    /// Words with this flag may start a compound
    pub compound_begin_flag: Option<String>,

    /// Words with this flag may end a compound
    pub compound_end_flag: Option<String>,

    /// Words with this flag may be in the middle of a compound
    pub compound_middle_flag: Option<String>,

    /// Words with this flag can't be on their own, only in compounds
    pub compound_only_flag: Option<String>,
    // There are lots of compound flags that haven't yet been implemented

    // ## Affix-related items
    pub input_conversions: Vec<Conversion>,

    pub output_conversions: Vec<Conversion>,

    // Rules for setting prefixes and suffixes
    pub affix_rules: Vec<Rule>,

    // Rules for suggestion replacements to try
    pub replacements: Vec<Conversion>,
}

impl Config {
    /// Create an empty affix object
    #[inline]
    pub const fn new() -> Self {
        Self {
            encoding: EncodingType::Utf8,
            complex_prefixes: false,
            lang: String::new(),
            ignore_chars: Vec::new(),
            afx_flag_vector: Vec::new(),
            keys: Vec::new(),
            try_characters: Vec::new(),
            nosuggest_flag: String::new(),
            compound_suggestions_max: 2,
            ngram_suggestions_max: 2,
            ngram_diff_max: 5,
            ngram_limit_to_diff_max: false,
            no_split_suggestions: false,
            keep_termination_dots: false,
            warn_rare_flag: String::new(),
            forbid_warn_words: false,
            compound_min_length: 3,
            compound_flag: None,
            compound_begin_flag: None,
            compound_end_flag: None,
            compound_middle_flag: None,
            compound_only_flag: None,
            input_conversions: Vec::new(),
            output_conversions: Vec::new(),
            affix_rules: Vec::new(),
            replacements: Vec::new(),
        }
    }

    /// Load this affix from a string, e.g. one read from an affix file
    ///
    /// # Errors
    ///
    /// Error if loading is unsuccessful
    #[inline]
    pub fn load_from_str(&mut self, s: &str) -> Result<(), AffixError> {
        load_affix_from_str(self, s)
    }

    /// Create a vector of words from a single root word by applying rules in
    /// this affix
    ///
    /// May contain duplicates
    ///
    /// # Parameters
    ///
    /// - `rootword`: The word to have prefixes/suffixes applied to
    /// - `keys`: Prefix and suffix keys to apply
    #[inline]
    pub fn create_affixed_words(&self, rootword: &str, keys: &str) -> Vec<String> {
        let mut ret = vec![rootword.to_owned()];
        // We will build our prefixed words here.
        let mut prefixed_words: Vec<String> = Vec::new();

        // List of what keys may apply to an affix
        let keys_vec: Vec<String> = graph_vec!(keys.to_uppercase());

        // Loop through rules where the identifiers apply
        // Then apply them
        self.affix_rules
            .iter()
            // Select rules whose identifier is in the desired keys
            .filter(|rule| keys_vec.contains(&rule.key))
            .for_each(|rule| {
                if let Some(newword) = rule.apply(rootword) {
                    if rule.combine_pfx_sfx && rule.atype == RuleType::Prefix {
                        prefixed_words.push(newword.clone());
                    }
                    ret.push(newword);
                }
            });

        // Redo the same thing for rules that allow chaining
        self.affix_rules
            .iter()
            // Select rules whose identifier is in the desired keys, and who
            // allow pfx+sfx combinations
            .filter(|rule| {
                rule.combine_pfx_sfx
                    && keys_vec.contains(&rule.key)
                    && rule.atype == RuleType::Suffix
            })
            .for_each(|rule| {
                for pfxword in &prefixed_words {
                    if let Some(newword) = rule.apply(pfxword) {
                        ret.push(newword);
                    }
                }
            });

        ret
    }
}

impl Default for Config {
    /// Common defaults for affix configuration with a QWERTY keyboard
    #[inline]
    fn default() -> Self {
        let mut ax = Self::new();

        ax.keys = vec![
            graph_vec!("qwertyuiop"),
            graph_vec!("asdfghjkl"),
            graph_vec!("zxcvbnm"),
        ];
        ax.try_characters = graph_vec!("esianrtolcdugmphbyfvkwzESIANRTOLCDUGMPHBYFVKWZ'");
        ax.nosuggest_flag = String::from("!");

        ax
    }
}
