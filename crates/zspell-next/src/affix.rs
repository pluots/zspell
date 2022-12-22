pub(crate) mod types;
pub(crate) mod types_impl;

use self::types::{CompoundSyllable, Conversion, Encoding, Flag, Phonetic, RuleGroup};

pub struct Config {
    /*
        General Options
    */
    /// Charset to use, reference to an [`EncodingType`] Currently this is
    /// unused; only UTF-8 is supported. However, the affix file must still have
    /// an accurate definition.
    encoding: Encoding,

    /// The type of flag in the `.dic` file
    flag: Flag,

    /// Twofold prefix skipping for e.g. right-to-left languages
    complex_prefixes: bool,

    /// Language code, currently unused. Consider this unstable as it may change
    /// to be an object reference.
    lang: String,

    /// List of characters to ignore
    ignore_chars: Vec<char>,

    /// List of usable flag vectors. Defaults to all things after "/"" in a dict.
    affix_alias: Vec<String>,

    /// List of usable flag vectors
    morph_alias: Vec<String>,

    /*
        Suggestion options
    */
    /// List of e.g. "qwerty", "asdfg" that define neighbors
    keys: Vec<String>,

    // Rules for suggestion replacements to try
    replacements: Vec<Conversion>,

    /// Suggest words that differe by 1 try character
    try_characters: String,

    /// Flag used to indicate words that should not be suggested
    nosuggest_flag: Option<char>,

    /// Maximum compound word suggestions
    compound_suggestions_max: u16,

    /// Note rare (i.e. commonly misspelled) words with this flag
    warn_rare_flag: Option<char>,

    /// Don't suggest anything with spaces
    no_split_suggestions: bool,

    /// If a dot comes with the spellcheck, return one with a suggestion word
    keep_termination_dots: bool,

    /// Whether to never suggest words with the warn flag (above)
    forbid_warn_words: bool,

    /// Replace commonly misused letters, e.g. `u`/`ü`
    maps: Vec<(char, char)>,

    /// Phonetic replacements for similar words
    phonetics: Vec<Phonetic>,

    /*
        ngram configuration
    */
    /// Max number of ngram suggestions
    ngram_suggestions_max: u16,

    /// N-gram similarity limit
    ngram_diff_max: u8,

    /// Remove all suggestions except the diff max
    ngram_limit_to_diff_max: bool,

    /*
        Compounding-related items

        Still quite a few missing from this section
    */
    /// Something like `-` to indicate whether both sides should be checked
    /// Prefer COMPOUNDRULE instead
    break_separators: Vec<String>,

    /// Regex-like rules for compound words
    compound_rules: Vec<String>,

    /// Minimum length of words used in a compound
    compound_min_length: u16,

    /// Words with this flag may be in compounds
    compound_flag: Option<char>,

    /// Words with this flag may start a compound
    compound_begin_flag: Option<char>,

    /// Words with this flag may end a compound
    compound_end_flag: Option<char>,

    /// Words with this flag may be in the middle of a compound
    compound_middle_flag: Option<char>,

    /// Words with this flag can't be on their own, only in compounds
    compound_only_flag: Option<char>,

    /// Allow these words inside compounds
    compound_permit_flag: Option<char>,
    compound_forbig_flag: Option<char>,
    compound_more_suffixes: bool,
    compound_root: Option<char>,
    compound_word_max: u16,
    compound_forbid_duplication: bool,
    compound_forbid_repeat: bool,
    compound_check_case: bool,
    compound_check_triple: bool,
    compound_simplify_triple: bool,
    compound_force_upper_flag: Option<char>,
    compound_syllable: CompoundSyllable,

    /*
        Affix Options
    */
    // Rules for setting prefixes and suffixes
    affix_rules: Vec<RuleGroup>,

    /*
        Other options
    */
    afx_circumflex_flag: Option<char>,
    forbidden_word_flag: Option<char>,
    fullstrip: bool,
    afx_keep_case_flag: char,
    input_conversions: Vec<Conversion>,
    output_conversions: Vec<Conversion>,
    afx_needed_flag: char,
    afx_substandard_flag: char,
    afx_word_chars: String,
    afx_check_sharps: bool,
}
